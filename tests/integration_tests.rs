use std::cell::RefCell;

mod common;
use common::SideFlux;

use flux_rs::my_future::MyFuture;
use flux_rs::Flux;

#[test]
fn test_00_sideflux() {
    let size = 4;
    let mut side_flux = SideFlux::start(4);
    {
        let child_status = side_flux
            .child
            .try_wait()
            .expect("Failed to get Flux child process status");
        assert!(child_status.is_none());
    }

    let mut h = Flux::open(&side_flux.uri, 0).expect("Failed to create handle to sideflux");

    let size_str = h.attr_get("size").expect("Failed to get size attr");
    let actual_size = size_str.parse::<u32>().expect("Failed to parse size");
    assert_eq!(size, actual_size);
}

#[test]
fn test_01_composite_future() {
    let side_flux = SideFlux::start(1);
    let mut h = Flux::open(&side_flux.uri, 0).expect("Failed to create handle");
    let eventlog = RefCell::<String>::new("".to_string());
    let hwloc_byrank = RefCell::<String>::new("".to_string());

    let _composite = h
        .kvs_lookup(0, "resource.eventlog")
        .expect("Failed to create kvs_lookup")
        .and_then(|fi| {
            eventlog.replace(fi.lookup_get().unwrap().into_string().unwrap());
            h.kvs_lookup(0, "resource.R")
        })
        .unwrap()
        .then(|f| {
            // f is strongly typed with kvs future methods
            hwloc_byrank.replace(f.lookup_get().unwrap().into_string().unwrap());
            flux_rs::reactor_stop(&mut h);
        })
        .unwrap();

    flux_rs::reactor_run(&mut h, 0).expect("Reactor running failed");

    assert!(eventlog.borrow().len() > 1);
}
