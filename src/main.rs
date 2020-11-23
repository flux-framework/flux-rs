use flux_sys;
use flux_rs::Flux;
use std::error::Error;
use flux_rs::my_future::MyFuture;

fn main() -> Result<(), Box<dyn Error>> {
    eprintln!("starting");
    let mut h = Flux::open("", 0)?;
    // h.service_register("sched")?.get()?;
    eprintln!("got a handle!");
    eprintln!("Hello, world! size:{:?}", h.attr_get("size")?);
    let mut composite = h
        .kvs_lookup(0, "resource.eventlog")?
        .and_then(|fi| {
            eprintln!("kvs result:{:?}", fi.lookup_get()?);
            h.kvs_lookup(0, "resource.hwloc.by_rank")
        })?
        .then(|f| {
            // f is strongly typed with kvs future methods
            println!("kvs result2:{:?}", f.lookup_get().unwrap());
            flux_rs::reactor_stop(&mut h);
        })?;
    composite.wait_for(-1.0)?;
    unsafe {
        flux_sys::flux_reactor_run(flux_sys::flux_get_reactor(h.get_handle_mut()), 0);
    };
    Ok(())
}
