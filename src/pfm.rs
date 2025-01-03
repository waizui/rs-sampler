use crate::envmap::tex2pixel;
use anyhow::anyhow;
use std::fs::File;
use std::io::BufReader;

type Real = f32;

pub struct PFM {
    pub data: Vec<f32>,
    pub w: usize,
    pub h: usize,
    pub channels: usize,
    pub little_endian: bool,
}

impl PFM {
    pub fn read_from(path: &str) -> anyhow::Result<PFM> {
        use std::io::BufRead;
        use std::io::Read;
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        let mut header = String::new();
        reader.read_line(&mut header)?;
        let header = header.trim();

        let channels = match header {
            "PF" => 3, // color image
            "Pf" => 1, // grayscale image
            _ => return Err(anyhow!("Invalid header of PFM")),
        };

        let mut dimensions = String::new();
        reader.read_line(&mut dimensions)?;
        let dims: Vec<&str> = dimensions.split_whitespace().collect();

        if dims.len() != 2 {
            return Err(anyhow!("Invalid dimensions format"));
        }

        let w: usize = dims[0]
            .parse()
            .map_err(|_| anyhow!("Invalid width: {}", dims[0]))?;
        let h: usize = dims[1]
            .parse()
            .map_err(|_| anyhow!("Invalid height: {}", dims[1]))?;

        // scale factor, little endian if scale is negative
        let mut scale_line = String::new();
        reader.read_line(&mut scale_line)?;
        let scale: f32 = scale_line
            .trim()
            .parse()
            .map_err(|_| anyhow!("Invalid scale factor: {}", scale_line.trim()))?;

        let little_endian = scale < 0.0;

        // binary data
        let data_size = w * h * channels;
        let mut buffer = vec![0u8; data_size * 4]; // 4 bytes per f32
        reader.read_exact(&mut buffer)?;

        // convert bytes to f32 values
        let mut data = Vec::with_capacity(data_size);
        for chunk in buffer.chunks_exact(4) {
            let bytes = [chunk[0], chunk[1], chunk[2], chunk[3]];
            let value = if little_endian {
                f32::from_le_bytes(bytes)
            } else {
                f32::from_be_bytes(bytes)
            };
            data.push(value);
        }

        Ok(PFM {
            data,
            w,
            h,
            channels,
            little_endian: true,
        })
    }

    pub fn save_to(&self, path: &str) -> anyhow::Result<()> {
        use std::io::Write;
        let mut file = File::create(path)?;
        let header = match self.channels {
            3 => "PF\n",
            1 => "Pf\n",
            _ => return Err(anyhow!("Invalid number of channels")),
        };

        file.write_all(header.as_bytes())?;
        let dimensions = format!("{} {}\n", self.w, self.h);
        file.write_all(dimensions.as_bytes())?;

        let scale = if self.little_endian { -1.0 } else { 1.0 };
        file.write_all(format!("{}\n", scale).as_bytes())?;

        for value in &self.data {
            let bytes = if self.little_endian {
                value.to_le_bytes()
            } else {
                value.to_be_bytes()
            };
            file.write_all(&bytes)?;
        }

        Ok(())
    }

    pub fn get_color(&self, u: Real, v: Real) -> [Real; 3] {
        let channel = self.channels;
        let i_w = tex2pixel(u, self.w);
        let i_h = tex2pixel(v, self.h);

        let i = channel * (i_h * self.w + i_w);
        if channel == 3 {
            let r = self.data[i];
            let g = self.data[i + 1];
            let b = self.data[i + 2];
            [r, g, b]
        } else if channel == 1 {
            let g = self.data[i];
            [g; 3]
        } else {
            panic!("Invalid channel");
        }
    }

    pub fn set_color(&mut self, col: &[Real; 3], u: Real, v: Real) {
        let channel = self.channels;
        let i_w = tex2pixel(u, self.w);
        let i_h = tex2pixel(v, self.h);

        let i = channel * (i_h * self.w + i_w);
        if channel == 3 {
            self.data[i] = col[0];
            self.data[i + 1] = col[1];
            self.data[i + 2] = col[2];
        } else if channel == 1 {
            self.data[i] = col[0];
        } else {
            panic!("Invalid channel");
        }
    }

    pub fn create_image(channels: usize, w: usize, h: usize) -> PFM {
        PFM {
            data: vec![0.; w * h * channels],
            w,
            h,
            channels,
            little_endian: false,
        }
    }
}
