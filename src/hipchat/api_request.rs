use super::{Controller, ApiResponse};
use hyper::header::Authorization;
use std::result::Result;
use hyper::client::Response;
use hyper;
use std::{self, io, cmp, thread};
use chrono::{DateTime, UTC};

header! { (XRatelimitReset, "X-Ratelimit-Reset") => [u64] }
header! { (XRatelimitRemaining, "X-RateLimit-Remaining") => [u16] }

pub struct ApiRequest {
    attempts: u8,
    max_attempts: u8,
    url: String,
    request_limit_remaining: u16,
    request_limit_reset: u64,
}

impl ApiRequest {
    pub fn new(url: String) -> ApiRequest {
        ApiRequest {
            attempts: 0,
            max_attempts: 5,
            url: url,
            request_limit_remaining: 0,
            request_limit_reset: 0,
        }
    }

    pub fn from_past_request(url: String, old_request: &ApiRequest) -> ApiRequest {
        ApiRequest {
            attempts: 0,
            max_attempts: 5,
            url: url,
            request_limit_remaining: old_request.request_limit_remaining,
            request_limit_reset: old_request.request_limit_reset,
        }
    }

    pub fn get_url(&self) -> &str {
        &self.url
    }

    fn register_failed_attempt(&mut self, message: &str) {
        self.attempts += 1;
        println!("Retrying attempt {}: {}", self.attempts, message);
    }
    
    fn single_attempt_request(&self, controller: &Controller) -> Result<Response, hyper::Error> {
        self.sleep_if_at_limit();
        controller.hyper.get(&self.url)
            .header(Authorization(controller.auth.clone()))
            .send()
    }

    fn multiple_attempt_request(&mut self, controller: &Controller) -> Result<Response, String> {
        while self.attempts < self.max_attempts {
            match self.single_attempt_request(controller) {
                Ok(res) => return Ok(res),
                Err(hyper::error::Error::Io(e)) => {
                    if e.kind() == io::ErrorKind::ConnectionAborted {
                        self.register_failed_attempt("Connection Aborted");
                    } else {
                        self.register_failed_attempt(&format!("Unknown error {:?}", e));
                    }
                    continue;
                },
                Err(e) => {
                    self.register_failed_attempt(&format!("Unknown error {:?}", e));
                }
            }
        }
        Err("Ran out of retry attempts".to_owned())
    }

    fn sleep_if_at_limit(&self) {
        if self.request_limit_remaining <= 1 { // HipChat's API lies and tells you that you have 1 request remaining when you don't.
            let now: u64 = UTC::now().timestamp() as u64;
            if now > self.request_limit_reset {
                // Time has already elapsed
                return;
            }
            let seconds_to_wait = cmp::max(self.request_limit_reset - now + 30, 10); // Add 30 seconds in case clocks are off
            println!("Hitting rate limit, sleeping for {} seconds", seconds_to_wait);
            thread::sleep(std::time::Duration::from_secs(seconds_to_wait));
        }
    }

    fn update_limits_from_response(&mut self, response: &Response) {
        match response.headers.get::<XRatelimitReset>() {
            Some(&XRatelimitReset(timestamp)) => { // in seconds
                let now: DateTime<UTC> = UTC::now();
                if timestamp > now.timestamp() as u64 {
                    self.request_limit_reset = timestamp;
                }
            },
            None => (),
        }
        
        match response.headers.get::<XRatelimitRemaining>() {
            Some(&XRatelimitRemaining(count)) => {
                self.request_limit_remaining = count;
            },
            None => (),
        }
    }

    fn request_check_rate_limit_response(&mut self, controller: &Controller) -> Result<Response, String> {
        while self.attempts < self.max_attempts {
            match self.multiple_attempt_request(controller) {
                Ok(res) => {
                    self.update_limits_from_response(&res);
                    match res.status {
                        hyper::status::StatusCode::TooManyRequests => {
                            println!("Too many requests returned, should hit sleep for rate limit next");
                            self.request_limit_remaining = 0;
                            // Loop again so it will hit the sleep
                            // before firing next request since we
                            // already updated the limits
                            continue;
                        }
                        hyper::Ok => {
                            return Ok(res);
                        }
                        _ => {
                            return Err(format!("Unknown status code {}", res.status));
                        }
                    }
                },
                Err(e) => {
                    // Ran out of retry attempts entirely at this
                    // point
                    return Err(e);
                }
            }
        }
        Err("Ran out of retry attempts".to_owned())
    }

    pub fn send(mut self, controller: &Controller) -> Result<ApiResponse, String> {
        match self.request_check_rate_limit_response(controller) {
            Ok(response) => {
                assert_eq!(response.status, hyper::Ok);
                Ok(ApiResponse::new(self, response))
            },
            Err(msg) => {
                Err(msg)
            }
        }
    }
}
