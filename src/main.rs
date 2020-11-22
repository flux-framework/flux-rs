use flux_sys;
use flux_rs::Flux;

fn main() -> Result<()> {
    eprintln!("starting");
    let mut h = Flux::open("", 0)?;
    // h.service_register("sched")?.get()?;
    eprintln!("got a handle!");
    eprintln!("Hello, world! size:{:?}", h.attr_get("size")?);
    let mut composite = h
        .kvs_lookup(0, "thing")?
        .and_then(|fi| {
            eprintln!("kvs result:{:?}", fi.lookup_get()?);
            h.kvs_lookup(0, "other_thing")
        })?
        .and_then(|fi| {
            eprintln!("kvs result2:{:?}", fi.lookup_get()?);
            h.kvs_lookup(0, "other_thing")
        })?
        .then(|f| {
            // f is strongly typed with kvs future methods
            println!("kvs result3:{:?}", f.lookup_get().unwrap());
        })?;
    composite.wait_for(-1.0)?;
    unsafe {
        flux_sys::flux_reactor_run(flux_sys::flux_get_reactor(h.handle), 0);
    };
    Ok(())
}
