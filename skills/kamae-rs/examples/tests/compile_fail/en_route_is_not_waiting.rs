use kamae_rs_taxi_request::{
    DriverAssignment, DriverId, PassengerId, RequestId, WaitingRequest,
};

fn only_waiting(_: WaitingRequest) {}

fn main() {
    let waiting = WaitingRequest::new(
        RequestId::new("req-1").unwrap(),
        PassengerId::new("passenger-1").unwrap(),
        false,
    );
    let driver = DriverAssignment::new(DriverId::new("driver-1").unwrap(), false);
    let transition = waiting
        .assign_driver(driver)
        .expect("fixture transition succeeds");

    only_waiting(transition.state);
}
