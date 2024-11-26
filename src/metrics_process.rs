use metrics::Unit;
use metrics::{describe_gauge, gauge};
use std::time::Duration;

use sysinfo::{get_current_pid, Disks, Networks, System};

use crate::round_up_f64_2digits;

const REFRESH_INTERVAL_SECONDS: u64 = 2;

pub const SYSTEM_CPU_CORE: &str = "system_cpu_core";
pub const SYSTEM_CPU_USAGE: &str = "system_cpu_usage";
pub const SYSTEM_MEMORY_USAGE: &str = "system_memory_usage";
pub const SYSTEM_MEMORY_TOTAL: &str = "system_memory_total";
pub const SYSTEM_SWAP_USAGE: &str = "system_swap_usage";
pub const SYSTEM_SWAP_TOTAL: &str = "system_swap_total";
pub const SYSTEM_DISK_USAGE: &str = "system_disk_usage";
pub const SYSTEM_NETWORK_UP_COUNT: &str = "system_network_up_count";
pub const SYSTEM_NETWORK_UP_SPEED: &str = "system_network_up_speed";
pub const SYSTEM_NETWORK_DOWN_COUNT: &str = "system_network_down_count";
pub const SYSTEM_NETWORK_DOWN_SPEED: &str = "system_network_down_speed";

pub const PROCESS_CPU_USAGE: &str = "process_cpu_utilization";
pub const PROCESS_MEMORY_USAGE: &str = "process_memory_usage";

pub fn register_sysinfo_event() {
    describe_gauge!(SYSTEM_CPU_CORE, "System CPU cores number");
    describe_gauge!(SYSTEM_CPU_USAGE, Unit::Percent, "System CPU usage");
    describe_gauge!(
        SYSTEM_MEMORY_USAGE,
        Unit::Percent,
        "System Memory usage in %"
    );
    describe_gauge!(SYSTEM_MEMORY_TOTAL, Unit::Bytes, "System Memory total");
    describe_gauge!(SYSTEM_SWAP_USAGE, Unit::Percent, "System Swap usage");
    describe_gauge!(SYSTEM_SWAP_TOTAL, Unit::Bytes, "System Swap total");
    describe_gauge!(SYSTEM_DISK_USAGE, Unit::Bytes, "System disk usage");

    describe_gauge!(SYSTEM_NETWORK_UP_COUNT, Unit::Bytes, "System network up");
    describe_gauge!(
        SYSTEM_NETWORK_UP_SPEED,
        Unit::BitsPerSecond,
        "System network up speed"
    );
    describe_gauge!(
        SYSTEM_NETWORK_DOWN_COUNT,
        Unit::Bytes,
        "System network down sum"
    );
    describe_gauge!(
        SYSTEM_NETWORK_DOWN_SPEED,
        Unit::BitsPerSecond,
        "System network down speed"
    );

    describe_gauge!(PROCESS_CPU_USAGE, Unit::Percent, "Process cpu usage");
    describe_gauge!(PROCESS_MEMORY_USAGE, Unit::Bytes, "Process memory usage");

    let pid = get_current_pid().expect("Should has");
    let mut sys = System::new_all();
    let mut disks = Disks::new();
    let mut networks = Networks::new();

    disks.refresh_list();
    networks.refresh_list();
    sys.refresh_all();
    sys.refresh_cpu_all();

    gauge!(SYSTEM_CPU_CORE).set(sys.cpus().len() as f64);

    let mut network_up_pre = 0;
    let mut network_down_pre = 0;

    std::thread::spawn(move || {
        loop {
            disks.refresh();
            networks.refresh();
            sys.refresh_all();
            sys.refresh_cpu_all();

            let mut sum = 0.0;
            for cpu in sys.cpus() {
                sum += cpu.cpu_usage() as f64;
            }
            gauge!(SYSTEM_CPU_USAGE).set(round_up_f64_2digits(sum / sys.cpus().len() as f64));

            gauge!(SYSTEM_MEMORY_TOTAL).set(sys.total_memory() as f64);
            gauge!(SYSTEM_MEMORY_USAGE).set(round_up_f64_2digits(
                100.0 * sys.used_memory() as f64 / sys.total_memory() as f64,
            ));
            gauge!(SYSTEM_SWAP_TOTAL).set(sys.total_swap() as f64);
            gauge!(SYSTEM_SWAP_USAGE).set(round_up_f64_2digits(
                100.0 * sys.used_swap() as f64 / sys.total_swap() as f64,
            ));

            let mut disk_used = 0.0;
            let mut disk_sum = 0.0;
            for disk in disks.iter() {
                disk_sum += disk.total_space() as f64;
                disk_used += (disk.total_space() - disk.available_space()) as f64;
            }

            gauge!(SYSTEM_DISK_USAGE).set(round_up_f64_2digits(100.0 * disk_used / disk_sum));

            let mut up_sum = 0;
            let mut down_sum = 0;
            for (_interface_name, data) in networks.iter() {
                up_sum += data.total_transmitted();
                down_sum += data.total_received();
            }

            if up_sum >= network_up_pre {
                gauge!(SYSTEM_NETWORK_UP_SPEED).set(round_up_f64_2digits(
                    8.0 * (up_sum - network_up_pre) as f64 / REFRESH_INTERVAL_SECONDS as f64,
                ));
            }

            if down_sum >= network_down_pre {
                gauge!(SYSTEM_NETWORK_DOWN_SPEED).set(round_up_f64_2digits(
                    8.0 * (down_sum - network_down_pre) as f64 / REFRESH_INTERVAL_SECONDS as f64,
                ));
            }

            gauge!(SYSTEM_NETWORK_UP_COUNT).set(up_sum as f64);
            gauge!(SYSTEM_NETWORK_DOWN_COUNT).set(up_sum as f64);

            network_down_pre = down_sum;
            network_up_pre = up_sum;

            // Process info
            if let Some(process) = sys.process(pid) {
                gauge!(PROCESS_CPU_USAGE).set(round_up_f64_2digits(process.cpu_usage() as f64));
                gauge!(PROCESS_MEMORY_USAGE).set(process.memory() as f64);
            }

            // Sleeping to let time for the system to run for long
            // enough to have useful information.
            std::thread::sleep(Duration::from_secs(REFRESH_INTERVAL_SECONDS));
        }
    });
}
