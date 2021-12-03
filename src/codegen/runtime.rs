struct TickEntry {
    ticks_left: u32,
    priority: u8,
    tick_fn: fn(&mut ContextObject),
}

pub struct ContextObject {
    to_be_ticked: Vec<TickEntry>,
}

pub extern "C" fn schedule_tick(ctx: &mut ContextObject) {}
