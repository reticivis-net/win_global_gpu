mod charging_events;
mod exe_scan;
mod winapp_scan;
mod registry;
mod notification;

fn unplug() {
    notification::toast("Unplugged");
}
fn plug() {
    notification::toast("Plugged in");
}

fn main() {
    println!("Hello, world!");
    unsafe { charging_events::register_events(unplug, plug); }
}
