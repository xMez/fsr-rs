use serialport::SerialPort;
use std::f64::consts::PI;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

// Serial communication function
pub async fn read_sensor_values(
    port: &Arc<Mutex<Box<dyn SerialPort>>>,
) -> Result<[i32; 4], Box<dyn std::error::Error + Send + Sync>> {
    let mut port_guard = port.lock().await;
    // Send the "v\n" command
    let output = "v\n".as_bytes();
    port_guard.write(output)?;

    // Read the response
    let mut serial_buf: Vec<u8> = Vec::with_capacity(23); // Max response size
    let mut buf = [0u8; 23];

    loop {
        match port_guard.read(&mut buf) {
            Ok(n) if n > 0 => {
                serial_buf.extend_from_slice(&buf[..n]);

                // Check if we have a complete line
                if let Some(pos) = serial_buf.iter().position(|&b| b == b'\n') {
                    serial_buf.truncate(pos + 1);
                    break;
                }
            }
            Ok(0) => {
                // No data read, continue immediately
                continue;
            }
            Ok(_) => continue,
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                // Timeout occurred, check if we have any data
                if !serial_buf.is_empty() {
                    // If we have partial data, break and use what we have
                    break;
                }
                return Err("Timeout reading sensor values".into());
            }
            Err(e) => return Err(Box::new(e)),
        }
    }

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
    port: &Arc<Mutex<Box<dyn SerialPort>>>,
    threshold_index: usize,
    value: i32,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut port_guard = port.lock().await;

    // Send the threshold command: "0 123\n" for threshold 0 with value 123
    let command = format!("{} {}\n", threshold_index, value);
    let output = command.as_bytes();
    port_guard.write(output)?;

    // Read the response
    let mut serial_buf: Vec<u8> = Vec::with_capacity(25); // Max response size for "t 123 1000 1000 1000\n"
    let mut buf = [0u8; 25];

    loop {
        match port_guard.read(&mut buf) {
            Ok(n) if n > 0 => {
                serial_buf.extend_from_slice(&buf[..n]);

                // Check if we have a complete line
                if let Some(pos) = serial_buf.iter().position(|&b| b == b'\n') {
                    serial_buf.truncate(pos + 1);
                    break;
                }
            }
            Ok(0) => {
                // No data read, continue immediately
                continue;
            }
            Ok(_) => continue,
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                // Timeout occurred, check if we have any data
                if !serial_buf.is_empty() {
                    // If we have partial data, break and use what we have
                    break;
                }
                return Err("Timeout reading threshold response".into());
            }
            Err(e) => return Err(Box::new(e)),
        }
    }

    // Parse the response: "t 123 1000 1000 1000\n"
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
    port: &Arc<Mutex<Box<dyn SerialPort>>>,
    thresholds: [i32; 4],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    for (index, &value) in thresholds.iter().enumerate() {
        set_threshold(port, index, value).await?;
    }
    Ok(())
}

// Function to read current thresholds from the serial device
pub async fn get_current_thresholds_from_device(
    port: &Arc<Mutex<Box<dyn SerialPort>>>,
) -> Result<[i32; 4], Box<dyn std::error::Error + Send + Sync>> {
    let mut port_guard = port.lock().await;

    // Send a command to get current thresholds (assuming "t\n" gets current thresholds)
    let command = "t\n".as_bytes();
    port_guard.write(command)?;

    // Read the response
    let mut serial_buf: Vec<u8> = Vec::with_capacity(25); // Max response size for "t 123 1000 1000 1000\n"
    let mut buf = [0u8; 25];

    loop {
        match port_guard.read(&mut buf) {
            Ok(n) if n > 0 => {
                serial_buf.extend_from_slice(&buf[..n]);

                // Check if we have a complete line
                if let Some(pos) = serial_buf.iter().position(|&b| b == b'\n') {
                    serial_buf.truncate(pos + 1);
                    break;
                }
            }
            Ok(0) => {
                // No data read, continue immediately
                continue;
            }
            Ok(_) => continue,
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                // Timeout occurred, check if we have any data
                if !serial_buf.is_empty() {
                    // If we have partial data, break and use what we have
                    break;
                }
                return Err("Timeout reading threshold values".into());
            }
            Err(e) => return Err(Box::new(e)),
        }
    }

    // Parse the response: "t 123 1000 1000 1000\n"
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

impl SerialPort for DummySerialPort {
    fn name(&self) -> Option<String> {
        Some("DUMMY".to_string())
    }

    fn baud_rate(&self) -> serialport::Result<u32> {
        Ok(115200)
    }

    fn data_bits(&self) -> serialport::Result<serialport::DataBits> {
        Ok(serialport::DataBits::Eight)
    }

    fn parity(&self) -> serialport::Result<serialport::Parity> {
        Ok(serialport::Parity::None)
    }

    fn stop_bits(&self) -> serialport::Result<serialport::StopBits> {
        Ok(serialport::StopBits::One)
    }

    fn flow_control(&self) -> serialport::Result<serialport::FlowControl> {
        Ok(serialport::FlowControl::None)
    }

    fn set_baud_rate(&mut self, _baud_rate: u32) -> serialport::Result<()> {
        Ok(())
    }

    fn set_data_bits(&mut self, _data_bits: serialport::DataBits) -> serialport::Result<()> {
        Ok(())
    }

    fn set_parity(&mut self, _parity: serialport::Parity) -> serialport::Result<()> {
        Ok(())
    }

    fn set_stop_bits(&mut self, _stop_bits: serialport::StopBits) -> serialport::Result<()> {
        Ok(())
    }

    fn set_flow_control(
        &mut self,
        _flow_control: serialport::FlowControl,
    ) -> serialport::Result<()> {
        Ok(())
    }

    fn set_timeout(&mut self, _timeout: Duration) -> serialport::Result<()> {
        Ok(())
    }

    fn timeout(&self) -> Duration {
        Duration::from_millis(100)
    }

    fn write_request_to_send(&mut self, _level: bool) -> serialport::Result<()> {
        Ok(())
    }

    fn write_data_terminal_ready(&mut self, _level: bool) -> serialport::Result<()> {
        Ok(())
    }

    fn read_clear_to_send(&mut self) -> serialport::Result<bool> {
        Ok(false)
    }

    fn read_data_set_ready(&mut self) -> serialport::Result<bool> {
        Ok(false)
    }

    fn read_ring_indicator(&mut self) -> serialport::Result<bool> {
        Ok(false)
    }

    fn read_carrier_detect(&mut self) -> serialport::Result<bool> {
        Ok(false)
    }

    fn bytes_to_read(&self) -> serialport::Result<u32> {
        Ok(0)
    }

    fn bytes_to_write(&self) -> serialport::Result<u32> {
        Ok(0)
    }

    fn clear(&self, _buffer_to_clear: serialport::ClearBuffer) -> serialport::Result<()> {
        Ok(())
    }

    fn try_clone(&self) -> serialport::Result<Box<dyn SerialPort>> {
        Ok(Box::new(DummySerialPort))
    }

    fn set_break(&self) -> serialport::Result<()> {
        Ok(())
    }

    fn clear_break(&self) -> serialport::Result<()> {
        Ok(())
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
    timeout: Duration,
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
            timeout: Duration::from_millis(100),
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

impl SerialPort for MockSerialPort {
    fn name(&self) -> Option<String> {
        Some("MOCK".to_string())
    }

    fn baud_rate(&self) -> serialport::Result<u32> {
        Ok(115200)
    }

    fn data_bits(&self) -> serialport::Result<serialport::DataBits> {
        Ok(serialport::DataBits::Eight)
    }

    fn parity(&self) -> serialport::Result<serialport::Parity> {
        Ok(serialport::Parity::None)
    }

    fn stop_bits(&self) -> serialport::Result<serialport::StopBits> {
        Ok(serialport::StopBits::One)
    }

    fn flow_control(&self) -> serialport::Result<serialport::FlowControl> {
        Ok(serialport::FlowControl::None)
    }

    fn set_baud_rate(&mut self, _baud_rate: u32) -> serialport::Result<()> {
        Ok(())
    }

    fn set_data_bits(&mut self, _data_bits: serialport::DataBits) -> serialport::Result<()> {
        Ok(())
    }

    fn set_parity(&mut self, _parity: serialport::Parity) -> serialport::Result<()> {
        Ok(())
    }

    fn set_stop_bits(&mut self, _stop_bits: serialport::StopBits) -> serialport::Result<()> {
        Ok(())
    }

    fn set_flow_control(
        &mut self,
        _flow_control: serialport::FlowControl,
    ) -> serialport::Result<()> {
        Ok(())
    }

    fn set_timeout(&mut self, timeout: Duration) -> serialport::Result<()> {
        self.timeout = timeout;
        Ok(())
    }

    fn timeout(&self) -> Duration {
        self.timeout
    }

    fn write_request_to_send(&mut self, _level: bool) -> serialport::Result<()> {
        Ok(())
    }

    fn write_data_terminal_ready(&mut self, _level: bool) -> serialport::Result<()> {
        Ok(())
    }

    fn read_clear_to_send(&mut self) -> serialport::Result<bool> {
        Ok(false)
    }

    fn read_data_set_ready(&mut self) -> serialport::Result<bool> {
        Ok(false)
    }

    fn read_ring_indicator(&mut self) -> serialport::Result<bool> {
        Ok(false)
    }

    fn read_carrier_detect(&mut self) -> serialport::Result<bool> {
        Ok(false)
    }

    fn bytes_to_read(&self) -> serialport::Result<u32> {
        Ok(self.read_buffer.len() as u32)
    }

    fn bytes_to_write(&self) -> serialport::Result<u32> {
        Ok(0)
    }

    fn clear(&self, _buffer_to_clear: serialport::ClearBuffer) -> serialport::Result<()> {
        Ok(())
    }

    fn try_clone(&self) -> serialport::Result<Box<dyn SerialPort>> {
        Ok(Box::new(MockSerialPort {
            thresholds: self.thresholds,
            read_buffer: self.read_buffer.clone(),
            timeout: self.timeout,
            phases: self.phases,
            phase_step: self.phase_step,
        }))
    }

    fn set_break(&self) -> serialport::Result<()> {
        Ok(())
    }

    fn clear_break(&self) -> serialport::Result<()> {
        Ok(())
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
