use flux_sys;
use flux_rs;
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
    flux_rs::reactor_run(&mut h, 0).map(|x| ()).map_err(|x| x.into())
}
