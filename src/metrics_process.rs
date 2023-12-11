use metrics::{describe_gauge, gauge};
use std::time::Duration;

use sysinfo::CpuExt;
use sysinfo::DiskExt;
use sysinfo::NetworkExt;

use sysinfo::ProcessExt;
use sysinfo::SystemExt;
use sysinfo::{get_current_pid, System};

use crate::round_up_f64_2digits;

const REFRESH_INTERVAL_SECONDS: u64 = 2;

const SYSTEM_CPU_CORE: &str = "system.cpu.core";
const SYSTEM_CPU_USAGE: &str = "system.cpu.usage";
const SYSTEM_MEMORY_USAGE: &str = "system.memory.usage";
const SYSTEM_MEMORY_TOTAL: &str = "system.memory.total";
const SYSTEM_SWAP_USAGE: &str = "system.swap.usage";
const SYSTEM_SWAP_TOTAL: &str = "system.swap.total";
const SYSTEM_DISK_USAGE: &str = "system.disk.usage";
const SYSTEM_NETWORK_UP_COUNT: &str = "system.network.up_count";
const SYSTEM_NETWORK_UP_SPEED: &str = "system.network.up_speed";
const SYSTEM_NETWORK_DOWN_COUNT: &str = "system.network.down_count";
const SYSTEM_NETWORK_DOWN_SPEED: &str = "system.network.down_speed";

const PROCESS_CPU_USAGE: &str = "process.cpu.utilization";
const PROCESS_MEMORY_USAGE: &str = "process.memory.usage";

pub fn register_sysinfo_event() {
    describe_gauge!(SYSTEM_CPU_CORE, "System CPU cores number");
    describe_gauge!(SYSTEM_CPU_USAGE, "System CPU usage in %");
    describe_gauge!(SYSTEM_MEMORY_USAGE, "System Memory usage in %");
    describe_gauge!(SYSTEM_MEMORY_TOTAL, "System Memory total in bytes");
    describe_gauge!(SYSTEM_SWAP_USAGE, "System Swap usage in %");
    describe_gauge!(SYSTEM_SWAP_TOTAL, "System Swap total in bytes");
    describe_gauge!(SYSTEM_DISK_USAGE, "System disk usage in bytes");

    describe_gauge!(SYSTEM_NETWORK_UP_COUNT, "System network up sum in bytes");
    describe_gauge!(SYSTEM_NETWORK_UP_SPEED, "System network up speed in bps");
    describe_gauge!(
        SYSTEM_NETWORK_DOWN_COUNT,
        "System network down sum in bytes"
    );
    describe_gauge!(
        SYSTEM_NETWORK_DOWN_SPEED,
        "System network down speed in bps"
    );

    describe_gauge!(PROCESS_CPU_USAGE, "Process cpu usage in %");
    describe_gauge!(PROCESS_MEMORY_USAGE, "Process memory usage in bytes");

    let pid = get_current_pid().expect("Should has");
    let mut sys = System::new_all();

    sys.refresh_all();
    sys.refresh_cpu();

    gauge!(SYSTEM_CPU_CORE, sys.cpus().len() as f64);

    let mut network_up_pre = 0;
    let mut network_down_pre = 0;

    std::thread::spawn(move || {
        loop {
            sys.refresh_all();
            sys.refresh_cpu();

            let mut sum = 0.0;
            for cpu in sys.cpus() {
                sum += cpu.cpu_usage() as f64;
            }
            gauge!(
                SYSTEM_CPU_USAGE,
                round_up_f64_2digits(sum / sys.cpus().len() as f64)
            );

            gauge!(SYSTEM_MEMORY_TOTAL, sys.total_memory() as f64);
            gauge!(
                SYSTEM_MEMORY_USAGE,
                round_up_f64_2digits(100.0 * sys.used_memory() as f64 / sys.total_memory() as f64)
            );
            gauge!(SYSTEM_SWAP_TOTAL, sys.total_swap() as f64);
            gauge!(
                SYSTEM_SWAP_USAGE,
                round_up_f64_2digits(100.0 * sys.used_swap() as f64 / sys.total_swap() as f64)
            );

            let mut disk_used = 0.0;
            let mut disk_sum = 0.0;
            for disk in sys.disks() {
                disk_sum += disk.total_space() as f64;
                disk_used += (disk.total_space() - disk.available_space()) as f64;
            }

            gauge!(
                SYSTEM_DISK_USAGE,
                round_up_f64_2digits(100.0 * disk_used / disk_sum)
            );

            let mut up_sum = 0;
            let mut down_sum = 0;
            for (_interface_name, data) in sys.networks() {
                up_sum += data.total_transmitted();
                down_sum += data.total_received();
            }

            if up_sum >= network_up_pre {
                gauge!(
                    SYSTEM_NETWORK_UP_SPEED,
                    round_up_f64_2digits(
                        8.0 * (up_sum - network_up_pre) as f64 / REFRESH_INTERVAL_SECONDS as f64
                    )
                );
            }

            if down_sum >= network_down_pre {
                gauge!(
                    SYSTEM_NETWORK_DOWN_SPEED,
                    round_up_f64_2digits(
                        8.0 * (down_sum - network_down_pre) as f64
                            / REFRESH_INTERVAL_SECONDS as f64
                    )
                );
            }

            gauge!(SYSTEM_NETWORK_UP_COUNT, up_sum as f64);
            gauge!(SYSTEM_NETWORK_DOWN_COUNT, up_sum as f64);

            network_down_pre = down_sum;
            network_up_pre = up_sum;

            // Process info
            if let Some(process) = sys.process(pid) {
                gauge!(
                    PROCESS_CPU_USAGE,
                    round_up_f64_2digits(process.cpu_usage() as f64)
                );
                gauge!(PROCESS_MEMORY_USAGE, process.memory() as f64);
            }

            // Sleeping to let time for the system to run for long
            // enough to have useful information.
            std::thread::sleep(Duration::from_secs(REFRESH_INTERVAL_SECONDS));
        }
    });
}
