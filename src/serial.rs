use serialport::SerialPort as ExternalSerialPort;
use std::f64::consts::PI;
use std::io::{Read, Write};
use std::sync::Arc;
use tokio::sync::Mutex;

// Serial communication function
// A minimal abstraction used by the app: only read/write are required.
pub trait SensorPort: Send {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize>;
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize>;
}

// Adapter to wrap a real serialport::SerialPort inside the minimal SensorPort API.
pub struct SerialPortAdapter {
    inner: Box<dyn ExternalSerialPort>,
}

impl SerialPortAdapter {
    pub fn new(inner: Box<dyn ExternalSerialPort>) -> Self {
        Self { inner }
    }
}

impl SensorPort for SerialPortAdapter {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        Read::read(&mut *self.inner, buf)
    }

    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        Write::write(&mut *self.inner, buf)
    }
}

// Maximum expected length of a single response line from the device.
// "t 1000 1000 1000 1000\n" is 22 bytes.
const MAX_LINE_CAPACITY: usize = 22;

fn read_serial_line(
    port: &mut dyn SensorPort,
    timeout_error_message: &'static str,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let mut serial_buf: Vec<u8> = Vec::with_capacity(MAX_LINE_CAPACITY);
    let mut chunk = vec![0u8; MAX_LINE_CAPACITY];

    loop {
        match port.read(&mut chunk) {
            Ok(n) if n > 0 => {
                serial_buf.extend_from_slice(&chunk[..n]);

                if let Some(pos) = serial_buf.iter().position(|&b| b == b'\n') {
                    serial_buf.truncate(pos + 1);
                    break;
                }
            }
            Ok(0) => continue,
            Ok(_) => continue,
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                if !serial_buf.is_empty() {
                    break;
                }
                return Err(timeout_error_message.into());
            }
            Err(e) => return Err(Box::new(e)),
        }
    }

    Ok(serial_buf)
}

pub async fn read_sensor_values(
    port: &Arc<Mutex<Box<dyn SensorPort>>>,
) -> Result<[i32; 4], Box<dyn std::error::Error + Send + Sync>> {
    let mut port_guard = port.lock().await;
    // Send the "v\n" command
    let output = "v\n".as_bytes();
    port_guard.write(output)?;

    // Read the response line
    let serial_buf = read_serial_line(&mut **port_guard, "Timeout reading sensor values")?;

    // Parse the response: "v 1000 1000 1000 1000\n"
    let response_str = String::from_utf8_lossy(&serial_buf);
    let parts: Vec<&str> = response_str.trim().split_whitespace().collect();

    if parts.len() != 5 || parts[0] != "v" {
        return Err("Invalid response format".into());
    }

    let mut values = [0i32; 4];
    for i in 0..4 {
        values[i] = parts[i + 1]
            .parse::<i32>()
            .map_err(|_| "Failed to parse sensor value")?;
    }

    Ok(values)
}

// Function to set threshold on serial device
pub async fn set_threshold(
    port: &Arc<Mutex<Box<dyn SensorPort>>>,
    threshold_index: usize,
    value: i32,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut port_guard = port.lock().await;

    // Send the threshold command: "0 123\n" for threshold 0 with value 123
    let command = format!("{} {}\n", threshold_index, value);
    let output = command.as_bytes();
    port_guard.write(output)?;

    // Read the response line
    let serial_buf = read_serial_line(&mut **port_guard, "Timeout reading threshold response")?;

    // Parse the response: "t 1000 1000 1000 1000\n"
    let response_str = String::from_utf8_lossy(&serial_buf);
    let parts: Vec<&str> = response_str.trim().split_whitespace().collect();

    if parts.len() != 5 || parts[0] != "t" {
        return Err("Invalid threshold response format".into());
    }

    // Validate that the correct threshold was set
    let set_threshold = parts[threshold_index + 1]
        .parse::<i32>()
        .map_err(|_| "Failed to parse threshold value")?;

    if set_threshold != value {
        return Err(format!(
            "Threshold validation failed: expected {}, got {}",
            value, set_threshold
        )
        .into());
    }

    Ok(())
}

// Function to set all thresholds for a profile on the serial device
pub async fn set_all_thresholds(
    port: &Arc<Mutex<Box<dyn SensorPort>>>,
    thresholds: [i32; 4],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    for (index, &value) in thresholds.iter().enumerate() {
        set_threshold(port, index, value).await?;
    }
    Ok(())
}

// Function to read current thresholds from the serial device
pub async fn get_current_thresholds_from_device(
    port: &Arc<Mutex<Box<dyn SensorPort>>>,
) -> Result<[i32; 4], Box<dyn std::error::Error + Send + Sync>> {
    let mut port_guard = port.lock().await;

    // Send a command to get current thresholds (assuming "t\n" gets current thresholds)
    let command = "t\n".as_bytes();
    port_guard.write(command)?;

    // Read the response line
    let serial_buf = read_serial_line(&mut **port_guard, "Timeout reading threshold values")?;

    // Parse the response: "t 1000 1000 1000 1000\n"
    let response_str = String::from_utf8_lossy(&serial_buf);
    let parts: Vec<&str> = response_str.trim().split_whitespace().collect();

    if parts.len() != 5 || parts[0] != "t" {
        return Err("Invalid threshold response format".into());
    }

    let mut thresholds = [0i32; 4];
    for i in 0..4 {
        thresholds[i] = parts[i + 1]
            .parse::<i32>()
            .map_err(|_| "Failed to parse threshold value")?;
    }

    Ok(thresholds)
}

// Dummy serial port for when the real one is not available
pub struct DummySerialPort;

impl SensorPort for DummySerialPort {
    fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotConnected,
            "Dummy serial port - no data available",
        ))
    }

    fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
        Ok(0)
    }
}

impl std::io::Read for DummySerialPort {
    fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotConnected,
            "Dummy serial port - no data available",
        ))
    }
}

impl std::io::Write for DummySerialPort {
    fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
        Ok(0) // Pretend we wrote everything
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

// Mock serial port that simulates a real device for development
pub struct MockSerialPort {
    thresholds: [i32; 4],
    read_buffer: Vec<u8>,
    phases: [f64; 4],
    phase_step: f64,
}

impl MockSerialPort {
    pub fn new(initial_thresholds: [i32; 4]) -> Self {
        // Phase offsets to differentiate channels
        let phases = [0.0, PI * 0.5, PI, PI * 1.5];
        // Roughly 0.2 Hz at ~60Hz polling → period ~5s
        let phase_step = 2.0 * PI * 0.2 / 60.0;
        Self {
            thresholds: initial_thresholds,
            read_buffer: Vec::new(),
            phases,
            phase_step,
        }
    }

    fn generate_sensor_values(&mut self) -> [i32; 4] {
        let mut values = [0i32; 4];
        for i in 0..4 {
            // Update phase and wrap around 2π
            self.phases[i] = (self.phases[i] + self.phase_step) % (2.0 * PI);
            let s = self.phases[i].sin(); // -1..1
            let v = ((s + 1.0) * 0.5 * 1023.0).round() as i32; // 0..1023
            values[i] = v;
        }
        values
    }

    fn enqueue_line(&mut self, line: String) {
        self.read_buffer.extend_from_slice(line.as_bytes());
    }
}

impl SensorPort for MockSerialPort {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.read_buffer.is_empty() {
            // No data queued; simulate non-blocking empty read
            return Ok(0);
        }
        let n = buf.len().min(self.read_buffer.len());
        let data = self.read_buffer.drain(..n).collect::<Vec<u8>>();
        buf[..n].copy_from_slice(&data);
        Ok(n)
    }

    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let s = std::str::from_utf8(buf).unwrap_or("");
        let line = s.trim();

        if line == "v" {
            let values = self.generate_sensor_values();
            self.enqueue_line(format!(
                "v {} {} {} {}\n",
                values[0], values[1], values[2], values[3]
            ));
        } else if line == "t" {
            self.enqueue_line(format!(
                "t {} {} {} {}\n",
                self.thresholds[0], self.thresholds[1], self.thresholds[2], self.thresholds[3]
            ));
        } else {
            // Expecting: "<index> <value>"
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() == 2 {
                if let (Ok(idx), Ok(val)) = (parts[0].parse::<usize>(), parts[1].parse::<i32>()) {
                    if idx < 4 {
                        self.thresholds[idx] = val;
                    }
                    self.enqueue_line(format!(
                        "t {} {} {} {}\n",
                        self.thresholds[0],
                        self.thresholds[1],
                        self.thresholds[2],
                        self.thresholds[3]
                    ));
                }
            }
        }

        Ok(buf.len())
    }
}

impl std::io::Read for MockSerialPort {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.read_buffer.is_empty() {
            // No data queued; simulate non-blocking empty read
            return Ok(0);
        }
        let n = buf.len().min(self.read_buffer.len());
        let data = self.read_buffer.drain(..n).collect::<Vec<u8>>();
        buf[..n].copy_from_slice(&data);
        Ok(n)
    }
}

impl std::io::Write for MockSerialPort {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let s = std::str::from_utf8(buf).unwrap_or("");
        let line = s.trim();

        if line == "v" {
            let values = self.generate_sensor_values();
            self.enqueue_line(format!(
                "v {} {} {} {}\n",
                values[0], values[1], values[2], values[3]
            ));
        } else if line == "t" {
            self.enqueue_line(format!(
                "t {} {} {} {}\n",
                self.thresholds[0], self.thresholds[1], self.thresholds[2], self.thresholds[3]
            ));
        } else {
            // Expecting: "<index> <value>"
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() == 2 {
                if let (Ok(idx), Ok(val)) = (parts[0].parse::<usize>(), parts[1].parse::<i32>()) {
                    if idx < 4 {
                        self.thresholds[idx] = val;
                    }
                    self.enqueue_line(format!(
                        "t {} {} {} {}\n",
                        self.thresholds[0],
                        self.thresholds[1],
                        self.thresholds[2],
                        self.thresholds[3]
                    ));
                }
            }
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_mock_port(initial: [i32; 4]) -> Arc<Mutex<Box<dyn SensorPort>>> {
        Arc::new(Mutex::new(
            Box::new(MockSerialPort::new(initial)) as Box<dyn SensorPort>
        ))
    }

    fn make_dummy_port() -> Arc<Mutex<Box<dyn SensorPort>>> {
        Arc::new(Mutex::new(Box::new(DummySerialPort) as Box<dyn SensorPort>))
    }

    #[tokio::test]
    async fn test_get_current_thresholds_with_mock() {
        let port = make_mock_port([10, 20, 30, 40]);
        let thresholds = get_current_thresholds_from_device(&port).await.unwrap();
        assert_eq!(thresholds, [10, 20, 30, 40]);
    }

    #[tokio::test]
    async fn test_set_threshold_with_mock() {
        let port = make_mock_port([10, 20, 30, 40]);
        set_threshold(&port, 2, 123).await.unwrap();
        let thresholds = get_current_thresholds_from_device(&port).await.unwrap();
        assert_eq!(thresholds[2], 123);
    }

    #[tokio::test]
    async fn test_set_all_thresholds_with_mock() {
        let port = make_mock_port([10, 20, 30, 40]);
        set_all_thresholds(&port, [1, 2, 3, 4]).await.unwrap();
        let thresholds = get_current_thresholds_from_device(&port).await.unwrap();
        assert_eq!(thresholds, [1, 2, 3, 4]);
    }

    #[tokio::test]
    async fn test_read_sensor_values_with_mock() {
        let port = make_mock_port([100, 200, 300, 400]);
        let values = read_sensor_values(&port).await.unwrap();
        assert_eq!(values.len(), 4);
        for v in values {
            assert!(v >= 0 && v <= 1023);
        }
    }

    #[tokio::test]
    async fn test_dummy_serial_errors() {
        let port = make_dummy_port();
        assert!(read_sensor_values(&port).await.is_err());
        assert!(get_current_thresholds_from_device(&port).await.is_err());
        assert!(set_threshold(&port, 0, 42).await.is_err());
    }
}
