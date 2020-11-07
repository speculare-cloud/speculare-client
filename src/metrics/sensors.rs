use crate::models;

use models::Sensors;
use psutil::sensors;
use psutil::sensors::TemperatureSensor;

/// Retrieve the sensors and return them as a Vec<String>
/// SLOW - This function take up to 180ms
/// TODO - Optimize or find alternative
pub fn get_sensors_data() -> Vec<Sensors> {
    let temperatures: Vec<TemperatureSensor> = sensors::temperatures()
        .into_iter()
        .filter_map(Result::ok)
        .collect();
    let mut sensors: Vec<Sensors> = Vec::with_capacity(temperatures.len());
    for temp in temperatures {
        sensors.push(Sensors {
            label: temp.label().unwrap_or("?").to_string(),
            temp: temp.current().celsius(),
        })
    }
    sensors
}
