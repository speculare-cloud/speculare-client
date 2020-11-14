/// Return the avg cpu_freq across all core as i64.
pub fn get_avg_cpufreq() -> i64 {
    match cpuid::clock_frequency() {
        Some(val) => val as i64,
        None => 0,
    }
}
