pub mod read_emf {
    use byteorder::{LittleEndian, ReadBytesExt};
    use encoding_rs::{UTF_8, UTF_16LE};
    use lazy_static::lazy_static;
    use serde::Serialize;
    use serde_json::to_string_pretty;
    use std::collections::HashMap;
    use std::fs::{self, File};
    use std::io::{BufReader, Cursor, Read};
    use std::path::Path;
    use thiserror::Error;

    pub fn read_file(file_path: &Path, out_dir: &Path) -> bool {
        let mut parser = EmfParser::new();
        match File::open(file_path) {
            Ok(file) => {
                let mut reader: BufReader<File> = BufReader::new(file);
                let mut buf = Vec::new();
                let _ = reader.read_to_end(&mut buf);

                let mut cursor = Cursor::new(&buf);

                // let header = EmfHeader::parse(&mut cursor)?;

                let mut records_count = 0;
                while cursor.position() < buf.len() as u64 {
                    let start_pos = cursor.position();
                    let record_type = cursor
                        .read_u32::<LittleEndian>()
                        .map_err(|e| e.to_string())
                        .unwrap();

                    let record_size = cursor
                        .read_u32::<LittleEndian>()
                        .map_err(|e| e.to_string())
                        .unwrap();

                    // println!("record_type {} record_size {}", record_type, record_size);

                    let _ = parser.parse_record(record_type, &buf);

                    cursor.set_position(start_pos + record_size as u64);
                    records_count += 1;
                }

                println!("records_count {} ", records_count);

                let json_str = match to_string_pretty(&parser.model) {
                    Ok(res) => res,
                    Err(_) => {
                        print!("Err json");
                        "No data".to_string()
                    }
                };
                let res_folder = out_dir.join(std::path::Path::new(&("jsons")));
                let _ = fs::create_dir(&res_folder);

                let fname = &file_path.file_name();

                let f_name_p = match &fname {
                    Some(name) => name,
                    None => {
                        println!("No name");
                        return false;
                    }
                };

                let fname = match f_name_p.to_str() {
                    Some(name) => name,
                    None => {
                        println!("No name");
                        return false;
                    }
                };

                fs::write(
                    res_folder.join(std::path::Path::new(&(fname.to_owned() + ".json"))),
                    json_str,
                )
                .expect("Unable to write file");
                return true;
            }
            Err(err) => {
                eprintln!(
                    "Ошибка открытия файла {:?}: {}",
                    file_path.file_name().unwrap().to_owned(),
                    err
                );
                return false;
            }
        }
    }

    #[derive(Debug, Error)]
    pub enum EmfParseError {
        #[error("Invalid record type: {0}")]
        InvalidRecordType(u32),
        #[error("Buffer too small for record")]
        BufferTooSmall,
        #[error("Data parsing error")]
        DataError(#[from] std::io::Error),
    }

    // Constants for various enums and mappings
    const COLOR_SPACE: &[(u32, &str)] = &[(1, "ENABLE"), (2, "DISABLE"), (3, "DELETE_TRANSFORM")];

    const COLOR_MATCH_TO_TARGET: &[(u32, &str)] = &[(0, "NOTEMBEDDED"), (1, "EMBEDDED")];

    const FLOOD_FILL: &[(u32, &str)] = &[(0, "Border"), (1, "Surface")];

    // Main parser structure
    pub struct EmfParser {
        model: TreeModel,
    }

    impl EmfParser {
        pub fn new() -> Self {
            EmfParser {
                model: TreeModel::new(),
            }
        }

        // Main parsing entry point
        pub fn parse_record(
            &mut self,
            record_type: u32,
            record_data: &[u8],
        ) -> Result<(), EmfParseError> {
            // Validate we have enough data for at least the base record header
            if record_data.len() < 8 {
                return Err(EmfParseError::BufferTooSmall);
            }

            // Get the appropriate parser function for this record type
            let parser_func = EMR_IDS
                .get(&record_type)
                .ok_or(EmfParseError::InvalidRecordType(record_type))?;

            // Parse the record size (first 4 bytes)
            let record_size = Cursor::new(&record_data).read_u32::<LittleEndian>()?;

            // Validate record size matches buffer size
            // if record_size as usize != record_data.len() {
            //     return Err(EmfParseError::BufferTooSmall);
            // }

            // Call the specific record parser
            parser_func(self, record_size as usize, record_data);

            Ok(())
        }

        // Helper method to add items to the tree model
        fn add_iter(
            &mut self,
            name: &str,
            value: String,
            offset: usize,
            length: usize,
            vtype: &str,
        ) {
            self.model.add_item(name, value, offset, length, vtype);
        }

        fn gc_begin_group(&mut self, _size: usize, value: &[u8]) {
            self.point_l(value, 0x14, "S");
            self.point_l(value, 0x1c, "E");

            let nlen = Cursor::new(&value[0x24..0x28])
                .read_u32::<LittleEndian>()
                .unwrap();
            self.add_iter("DescLength", nlen.to_string(), 0x24, 4, "<I");

            let text_bytes = &value[0x28..0x28 + (nlen * 2) as usize];
            let (txt, _, _) = UTF_16LE.decode(text_bytes);
            self.add_iter(
                "Description",
                txt.to_string(),
                0x28,
                (nlen * 2) as usize,
                "txt",
            );
        }

        fn gc_end_group(&mut self, _size: usize, _value: &[u8]) {
            // No operation needed
        }

        // Point parsing functions
        fn point_s(&mut self, data: &[u8], offset: usize, suffix: &str) {
            let x = read_i16(&data[offset..offset + 2]);
            let y = read_i16(&data[offset + 2..offset + 4]);
            self.add_iter(&format!("x{}", suffix), x.to_string(), offset, 2, "<h");
            self.add_iter(&format!("y{}", suffix), y.to_string(), offset + 2, 2, "<h");
        }

        fn point_l(&mut self, data: &[u8], offset: usize, suffix: &str) {
            let x = read_i32(&data[offset..offset + 4]);
            let y = read_i32(&data[offset + 4..offset + 8]);
            self.add_iter(&format!("x{}", suffix), x.to_string(), offset, 4, "<i");
            self.add_iter(&format!("y{}", suffix), y.to_string(), offset + 4, 4, "<i");
        }

        // Record parsing functions
        pub fn parse_header(&mut self, _size: usize, data: &[u8]) {
            println!("parse_header");
            self.point_l(data, 8, "S");
            self.point_l(data, 16, "E");
            self.point_l(data, 24, "S (mm)");
            self.point_l(data, 32, "E (mm)");

            let sig = read_i32(&data[40..44]);
            self.add_iter("Signature", format!("0x{:08X}", sig), 40, 4, "<i");

            let version = read_i32(&data[44..48]);
            self.add_iter("Version", format!("0x{:08X}", version), 44, 4, "<i");

            let size = read_u32(&data[48..52]);
            self.add_iter("Size", size.to_string(), 48, 4, "<I");

            let records = read_u32(&data[52..56]);
            self.add_iter("Records", records.to_string(), 52, 4, "<I");

            let objects = read_u16(&data[56..58]);
            self.add_iter("Objects", objects.to_string(), 56, 2, "<H");

            let reserved = read_u16(&data[58..60]);
            self.add_iter("Reservd", reserved.to_string(), 58, 2, "<H");

            let descsize = read_u32(&data[60..64]);
            self.add_iter("DescSize", descsize.to_string(), 60, 4, "<I");

            let descoff = read_u32(&data[64..68]);
            self.add_iter("DescOffset", format!("0x{:02x}", descoff), 64, 4, "<I");

            let palnum = read_u32(&data[68..72]);
            self.add_iter("PalEntries", palnum.to_string(), 68, 4, "<I");

            self.point_l(data, 72, "Dev");
            self.point_l(data, 80, "Dev (mm)");

            let cb_pxl_fmt = read_u32(&data[88..92]);
            self.add_iter("cbPxlFmt", cb_pxl_fmt.to_string(), 88, 4, "<I");

            let off_pxl_fmt = read_u32(&data[92..96]);
            self.add_iter("offPxlFmt", off_pxl_fmt.to_string(), 92, 4, "<I");

            let b_opengl = read_u32(&data[96..100]);
            self.add_iter("bOpenGL", b_opengl.to_string(), 96, 4, "<I");

            self.point_l(data, 100, " (micrometers)");

            let desc_bytes = &data[descoff as usize..(descoff + descsize * 2) as usize];
            let (desc, _, _) = UTF_16LE.decode(desc_bytes);
            self.add_iter(
                "Description",
                desc.to_string(),
                descoff as usize,
                (descsize * 2) as usize,
                "txt",
            );
        }

        pub fn polybezier(&mut self, size: usize, value: &[u8]) {
            println!("parse polybezier");
            // First parse the rectangle that contains the polybezier
            self.rectangle(size, value);

            // Get the count of points from bytes 24-28
            let count = Cursor::new(&value[24..28])
                .read_i32::<LittleEndian>()
                .unwrap();
            self.add_iter("Count", count.to_string(), 24, 4, "<i");

            // Parse each point (8 bytes per point)
            for i in 0..count {
                let offset = 28 + (i as usize * 8);
                self.point_l(value, offset, &i.to_string());
            }
        }

        /// Parses an EMF Rectangle record (Record type 0x2B)
        ///
        /// # Arguments
        /// * `size` - The total size of the record in bytes
        /// * `value` - The binary data containing the rectangle coordinates
        pub fn rectangle(&mut self, _size: usize, value: &[u8]) {
            println!("parse rectangle");
            // Parse and add x-coordinate of start point (8-12 bytes)
            let xs = Cursor::new(&value[8..12])
                .read_i32::<LittleEndian>()
                .expect("Failed to read xS coordinate");
            self.add_iter("xS", xs.to_string(), 8, 4, "<i");

            // Parse and add y-coordinate of start point (12-16 bytes)
            let ys = Cursor::new(&value[12..16])
                .read_i32::<LittleEndian>()
                .expect("Failed to read yS coordinate");
            self.add_iter("yS", ys.to_string(), 12, 4, "<i");

            // Parse and add x-coordinate of end point (16-20 bytes)
            let xe = Cursor::new(&value[16..20])
                .read_i32::<LittleEndian>()
                .expect("Failed to read xE coordinate");
            self.add_iter("xE", xe.to_string(), 16, 4, "<i");

            // Parse and add y-coordinate of end point (20-24 bytes)
            let ye = Cursor::new(&value[20..24])
                .read_i32::<LittleEndian>()
                .expect("Failed to read yE coordinate");
            self.add_iter("yE", ye.to_string(), 20, 4, "<i");
        }

        pub fn polygon(&mut self, size: usize, value: &[u8]) {
            self.polybezier(size, value);
        }

        // Record type 0x04 - Polyline
        pub fn polyline(&mut self, size: usize, value: &[u8]) {
            self.polybezier(size, value);
        }

        // Record type 0x05 - PolybezierTo
        pub fn polybezier_to(&mut self, size: usize, value: &[u8]) {
            self.polybezier(size, value);
        }

        // Record type 0x06 - PolylineTo
        pub fn polyline_to(&mut self, size: usize, value: &[u8]) {
            self.polybezier(size, value);
        }

        // Record type 0x07 - PolyPolyline
        pub fn poly_polyline(&mut self, size: usize, value: &[u8]) {
            self.rectangle(size, value);

            let numpoly = Cursor::new(&value[24..28])
                .read_i32::<LittleEndian>()
                .unwrap();
            self.add_iter("NumOfPoly", numpoly.to_string(), 24, 4, "<i");

            let count = Cursor::new(&value[28..32])
                .read_i32::<LittleEndian>()
                .unwrap();
            self.add_iter("Count", count.to_string(), 28, 4, "<i");

            for i in 0..numpoly {
                let offset = 32 + (i * 4) as usize;
                let val = Cursor::new(&value[offset..offset + 4])
                    .read_u32::<LittleEndian>()
                    .unwrap();
                self.add_iter(&format!("PolyPnt {}", i), val.to_string(), offset, 4, "<I");
            }

            for i in 0..count {
                let offset = 32 + (numpoly * 4) as usize + (i * 8) as usize;
                self.point_l(value, offset, &i.to_string());
            }
        }

        // Record type 0x08 - PolyPolygon
        pub fn poly_polygon(&mut self, size: usize, value: &[u8]) {
            self.poly_polyline(size, value);
        }

        // Record type 0x09 - SetWindowExtEx
        pub fn set_window_ext_ex(&mut self, _size: usize, value: &[u8]) {
            self.point_l(value, 8, "");
        }

        // Record type 0x0A - SetWindowOrgEx
        pub fn set_window_org_ex(&mut self, size: usize, value: &[u8]) {
            self.set_window_ext_ex(size, value);
        }

        // Record type 0x0B - SetViewportExtEx
        pub fn set_viewport_ext_ex(&mut self, size: usize, value: &[u8]) {
            self.set_window_ext_ex(size, value);
        }

        // Record type 0x0C - SetViewportOrgEx
        pub fn set_viewport_org_ex(&mut self, size: usize, value: &[u8]) {
            self.set_window_ext_ex(size, value);
        }

        // Record type 0x0D - SetBrushOrgEx
        pub fn set_brush_org_ex(&mut self, _size: usize, value: &[u8]) {
            self.point_l(value, 8, "Org");
        }

        // Record type 0x10 - SetMapperFlags
        pub fn set_mapper_flags(&mut self, _size: usize, value: &[u8]) {
            let mode = Cursor::new(&value[8..12])
                .read_u32::<LittleEndian>()
                .unwrap();
            self.add_iter("Mode", mode.to_string(), 8, 4, "<I");
        }

        // Record type 0x11 - SetMapMode
        pub fn set_map_mode(&mut self, size: usize, value: &[u8]) {
            self.set_bk_mode(size, value);
        }

        // Record type 0x12 - SetBKMode
        pub fn set_bk_mode(&mut self, _size: usize, value: &[u8]) {
            let mode = Cursor::new(&value[8..12])
                .read_u32::<LittleEndian>()
                .unwrap();
            self.add_iter("Mode", mode.to_string(), 8, 4, "<I");
        }

        // Record type 0x13 - SetPolyfillMode
        pub fn set_polyfill_mode(&mut self, size: usize, value: &[u8]) {
            self.set_bk_mode(size, value);
        }

        // Record type 0x14 - SetRop2
        pub fn set_rop2(&mut self, _size: usize, value: &[u8]) {
            let mode = Cursor::new(&value[8..12])
                .read_u32::<LittleEndian>()
                .unwrap();
            self.add_iter("Mode", format!("0x{:X}", mode), 8, 4, "<I");
        }

        // Record type 0x15 - SetStretchBltMode
        pub fn set_stretch_blt_mode(&mut self, size: usize, value: &[u8]) {
            self.set_bk_mode(size, value);
        }

        // Record type 0x16 - SetTextAlign
        pub fn set_text_align(&mut self, size: usize, value: &[u8]) {
            self.set_bk_mode(size, value);
        }

        pub fn set_color_adjustment(&mut self, _size: usize, value: &[u8]) {
            let size_val = Cursor::new(&value[8..10])
                .read_i16::<LittleEndian>()
                .unwrap();
            self.add_iter("Size", size_val.to_string(), 8, 2, "<i");

            let values = Cursor::new(&value[10..12])
                .read_i16::<LittleEndian>()
                .unwrap();
            self.add_iter("Values", values.to_string(), 10, 2, "<i");

            let illum_idx = Cursor::new(&value[12..14])
                .read_i16::<LittleEndian>()
                .unwrap();
            self.add_iter("IllumIdx", illum_idx.to_string(), 12, 2, "<i");

            let red_gamma = Cursor::new(&value[14..16])
                .read_i16::<LittleEndian>()
                .unwrap();
            self.add_iter("RedGamma", red_gamma.to_string(), 14, 2, "<i");

            let green_gamma = Cursor::new(&value[16..18])
                .read_i16::<LittleEndian>()
                .unwrap();
            self.add_iter("GreenGamma", green_gamma.to_string(), 16, 2, "<i");

            let blue_gamma = Cursor::new(&value[18..20])
                .read_i16::<LittleEndian>()
                .unwrap();
            self.add_iter("BlueGamma", blue_gamma.to_string(), 18, 2, "<i");

            let ref_black = Cursor::new(&value[20..22])
                .read_i16::<LittleEndian>()
                .unwrap();
            self.add_iter("RefBlack", ref_black.to_string(), 20, 2, "<i");

            let ref_white = Cursor::new(&value[22..24])
                .read_i16::<LittleEndian>()
                .unwrap();
            self.add_iter("RefWhite", ref_white.to_string(), 22, 2, "<i");

            let contrast = Cursor::new(&value[24..26])
                .read_i16::<LittleEndian>()
                .unwrap();
            self.add_iter("Contrast", contrast.to_string(), 24, 2, "<i");

            let brightness = Cursor::new(&value[26..28])
                .read_i16::<LittleEndian>()
                .unwrap();
            self.add_iter("Brightness", brightness.to_string(), 26, 2, "<i");

            let colorfull = Cursor::new(&value[28..30])
                .read_i16::<LittleEndian>()
                .unwrap();
            self.add_iter("Colorfull", colorfull.to_string(), 28, 2, "<i");

            let red_green_tint = Cursor::new(&value[30..32])
                .read_i16::<LittleEndian>()
                .unwrap();
            self.add_iter("RedGreenTint", red_green_tint.to_string(), 30, 2, "<i");
        }

        // Record type 0x18 - SetTextColor
        pub fn set_text_color(&mut self, size: usize, value: &[u8]) {
            self.set_bk_color(size, value);
        }

        // Record type 0x19 - SetBKColor
        pub fn set_bk_color(&mut self, _size: usize, value: &[u8]) {
            let clr = format!("{:02X}{:02X}{:02X}", value[10], value[9], value[8]);
            self.add_iter("RGB", clr, 8, 3, "clr");
        }

        // Record type 0x1A - OffsetClipRgn
        pub fn offset_clip_rgn(&mut self, size: usize, value: &[u8]) {
            self.set_window_ext_ex(size, value);
        }

        // Record type 0x1C - MoveToEx
        pub fn move_to_ex(&mut self, size: usize, value: &[u8]) {
            self.set_window_ext_ex(size, value);
        }

        // Record type 0x1D - ExcludeClipRect
        pub fn exclude_clip_rect(&mut self, size: usize, value: &[u8]) {
            self.rectangle(size, value);
        }

        // Record type 0x1E - IntersectClipRect
        pub fn intersect_clip_rect(&mut self, size: usize, value: &[u8]) {
            self.rectangle(size, value);
        }

        // Record type 0x1F - ScaleViewportExtEx
        pub fn scale_viewport_ext_ex(&mut self, _size: usize, value: &[u8]) {
            let x_num = Cursor::new(&value[8..12])
                .read_i32::<LittleEndian>()
                .unwrap();
            self.add_iter("xNum", x_num.to_string(), 8, 4, "<i");

            let x_denom = Cursor::new(&value[12..16])
                .read_i32::<LittleEndian>()
                .unwrap();
            self.add_iter("xDenom", x_denom.to_string(), 12, 4, "<i");

            let y_num = Cursor::new(&value[16..20])
                .read_i32::<LittleEndian>()
                .unwrap();
            self.add_iter("yNum", y_num.to_string(), 16, 4, "<i");

            let y_denom = Cursor::new(&value[20..24])
                .read_i32::<LittleEndian>()
                .unwrap();
            self.add_iter("yDenom", y_denom.to_string(), 20, 4, "<i");
        }

        // Record type 0x20 - ScaleWindowExtEx
        pub fn scale_window_ext_ex(&mut self, size: usize, value: &[u8]) {
            self.scale_viewport_ext_ex(size, value);
        }

        // Record type 0x21 - SaveDC
        pub fn save_dc(&mut self, _size: usize, _value: &[u8]) {
            // No operation needed
        }

        // Record type 0x22 - RestoreDC
        pub fn restore_dc(&mut self, _size: usize, value: &[u8]) {
            let saved_dc = Cursor::new(&value[8..12])
                .read_i32::<LittleEndian>()
                .unwrap();
            self.add_iter("SavedDC", saved_dc.to_string(), 8, 4, "<i");
        }

        // Record type 0x23 - SetWorldTransform
        pub fn set_world_transform(&mut self, _size: usize, value: &[u8]) {
            let m11 = Cursor::new(&value[8..12])
                .read_f32::<LittleEndian>()
                .unwrap();
            self.add_iter("m11", m11.to_string(), 8, 4, "<f");

            let m12 = Cursor::new(&value[12..16])
                .read_f32::<LittleEndian>()
                .unwrap();
            self.add_iter("m12", m12.to_string(), 12, 4, "<f");

            let m21 = Cursor::new(&value[16..20])
                .read_f32::<LittleEndian>()
                .unwrap();
            self.add_iter("m21", m21.to_string(), 16, 4, "<f");

            let m22 = Cursor::new(&value[20..24])
                .read_f32::<LittleEndian>()
                .unwrap();
            self.add_iter("m22", m22.to_string(), 20, 4, "<f");

            let dx = Cursor::new(&value[24..28])
                .read_f32::<LittleEndian>()
                .unwrap();
            self.add_iter("Dx", dx.to_string(), 24, 4, "<f");

            let dy = Cursor::new(&value[28..32])
                .read_f32::<LittleEndian>()
                .unwrap();
            self.add_iter("Dy", dy.to_string(), 28, 4, "<f");
        }

        // Record type 0x24 - ModifyWorldTransform
        pub fn modify_world_transform(&mut self, size: usize, value: &[u8]) {
            self.set_world_transform(size, value);

            let mode = Cursor::new(&value[32..36])
                .read_u32::<LittleEndian>()
                .unwrap();
            self.add_iter("Mode", mode.to_string(), 32, 4, "<I");
        }

        // Record type 0x25 - SelectObject
        pub fn select_object(&mut self, _size: usize, value: &[u8]) {
            let obj_id = Cursor::new(&value[8..12])
                .read_u32::<LittleEndian>()
                .unwrap();
            self.add_iter("ObjID", format!("0x{:X}", obj_id), 8, 4, "<I");
        }

        // Record type 0x26 - CreatePen
        pub fn create_pen(&mut self, _size: usize, value: &[u8]) {
            let obj_id = Cursor::new(&value[8..12])
                .read_u32::<LittleEndian>()
                .unwrap();
            self.add_iter("ObjID", format!("0x{:X}", obj_id), 8, 4, "<I");

            let pen_style = Cursor::new(&value[12..16])
                .read_i32::<LittleEndian>()
                .unwrap();
            self.add_iter("PenStyle", pen_style.to_string(), 12, 4, "<i");

            let width = Cursor::new(&value[16..20])
                .read_i32::<LittleEndian>()
                .unwrap();
            self.add_iter("Width", width.to_string(), 16, 4, "<i");

            let clr = format!("{:02X}{:02X}{:02X}", value[26], value[25], value[24]);
            self.add_iter("RGB", clr, 24, 3, "clr");
        }

        // Record type 0x27 - CreateBrushIndirect
        pub fn create_brush_indirect(&mut self, _size: usize, value: &[u8]) {
            let obj_id = Cursor::new(&value[8..12])
                .read_u32::<LittleEndian>()
                .unwrap();
            self.add_iter("ObjID", format!("0x{:X}", obj_id), 8, 4, "<I");

            let brush_style = Cursor::new(&value[12..16])
                .read_i32::<LittleEndian>()
                .unwrap();
            self.add_iter("BrushStyle", brush_style.to_string(), 12, 4, "<i");

            let clr = format!("{:02X}{:02X}{:02X}", value[18], value[17], value[16]);
            self.add_iter("RGB", clr, 16, 3, "clr");

            let hatch = Cursor::new(&value[20..24])
                .read_i32::<LittleEndian>()
                .unwrap();
            self.add_iter("Hatch", hatch.to_string(), 20, 4, "<i");
        }

        // Record type 0x28 - DeleteObject
        pub fn delete_object(&mut self, size: usize, value: &[u8]) {
            self.select_object(size, value);
        }

        // Record type 0x29 - AngleArc
        pub fn angle_arc(&mut self, _size: usize, value: &[u8]) {
            self.point_l(value, 8, "C");

            let radius = Cursor::new(&value[16..20])
                .read_u32::<LittleEndian>()
                .unwrap();
            self.add_iter("Radius", radius.to_string(), 16, 4, "<I");

            let start_ang = Cursor::new(&value[20..24])
                .read_f32::<LittleEndian>()
                .unwrap();
            self.add_iter("StartAng", start_ang.to_string(), 20, 4, "<f");

            let sweep_ang = Cursor::new(&value[24..28])
                .read_f32::<LittleEndian>()
                .unwrap();
            self.add_iter("SweepAng", sweep_ang.to_string(), 24, 4, "<f");
        }

        // Record type 0x2A - Ellipse
        pub fn ellipse(&mut self, size: usize, value: &[u8]) {
            self.rectangle(size, value);
        }

        // Record type 0x2C - RoundRect
        pub fn round_rect(&mut self, size: usize, value: &[u8]) {
            self.rectangle(size, value);
            self.point_l(value, 24, "R");
        }

        // Record type 0x2D - Arc
        pub fn arc(&mut self, size: usize, value: &[u8]) {
            self.rectangle(size, value);
            self.point_l(value, 24, "S");
            self.point_l(value, 32, "E");
        }

        // Record type 0x2E - Chord
        pub fn chord(&mut self, size: usize, value: &[u8]) {
            self.arc(size, value);
        }

        // Record type 0x2F - Pie
        pub fn pie(&mut self, size: usize, value: &[u8]) {
            self.arc(size, value);
        }

        // Record type 0x30 - SelectPalette
        pub fn select_palette(&mut self, size: usize, value: &[u8]) {
            self.select_object(size, value);
        }

        // Record type 0x33 - ResizePalette
        pub fn resize_palette(&mut self, _size: usize, value: &[u8]) {
            let lh_pal = Cursor::new(&value[8..12])
                .read_i32::<LittleEndian>()
                .unwrap();
            self.add_iter("lhPal", lh_pal.to_string(), 8, 4, "<i");

            let num_entries = Cursor::new(&value[12..16])
                .read_i32::<LittleEndian>()
                .unwrap();
            self.add_iter("NumOfEntries", num_entries.to_string(), 12, 4, "<i");
        }

        /// Parses an EMF ExtFloodFill record (Record type 0x35)
        ///
        /// # Arguments
        /// * `size` - The total size of the record in bytes
        /// * `value` - The binary data containing the flood fill parameters
        pub fn ext_flood_fill(&mut self, _size: usize, value: &[u8]) {
            // Parse and add the starting point (8 bytes into the record)
            self.point_l(value, 8, "Start");

            // Parse and add the fill color (0x10-0x14 bytes)
            let color = Cursor::new(&value[0x10..0x14])
                .read_u32::<LittleEndian>()
                .expect("Failed to read flood fill color");
            self.add_iter("Color", color.to_string(), 0x10, 4, "<I");

            // Parse and add the flood fill mode (0x14-0x18 bytes)
            let fill_mode = Cursor::new(&value[0x14..0x18])
                .read_u32::<LittleEndian>()
                .expect("Failed to read flood fill mode");

            // Look up the fill mode description
            let fill_desc = FLOOD_FILL
                .iter()
                .find(|&&(mode, _)| mode == fill_mode)
                .map(|&(_, desc)| desc)
                .unwrap_or("unknown");

            self.add_iter(
                "FloodFillMode",
                format!("{} ({})", fill_mode, fill_desc),
                0x14,
                4,
                "<I",
            );
        }

        // Record type 0x36 - LineTo
        pub fn line_to(&mut self, size: usize, value: &[u8]) {
            self.set_window_ext_ex(size, value);
        }

        // Record type 0x37 - ArcTo
        pub fn arc_to(&mut self, size: usize, value: &[u8]) {
            self.arc(size, value);
        }

        // Record type 0x38 - PolyDraw
        pub fn polydraw(&mut self, size: usize, value: &[u8]) {
            self.polybezier(size, value);

            let count = Cursor::new(&value[24..28])
                .read_i32::<LittleEndian>()
                .unwrap();
            self.add_iter("Count", count.to_string(), 24, 4, "<i");

            for i in 0..count {
                let offset = (count * 4 + 28 + i) as usize;
                let ab_type = value[offset];
                self.add_iter(
                    &format!("abType {}", i),
                    ab_type.to_string(),
                    offset,
                    1,
                    "b",
                );
            }
        }

        // Record type 0x39 - SetArcDirection
        pub fn set_arc_direction(&mut self, _size: usize, value: &[u8]) {
            let direction = Cursor::new(&value[8..12])
                .read_u32::<LittleEndian>()
                .unwrap();
            self.add_iter("ArcDirection", direction.to_string(), 8, 4, "<I");
        }

        // Record type 0x3A - SetMiterLimit
        pub fn set_miter_limit(&mut self, _size: usize, value: &[u8]) {
            let limit = Cursor::new(&value[8..12])
                .read_u32::<LittleEndian>()
                .unwrap();
            self.add_iter("MiterLimit", limit.to_string(), 8, 4, "<I");
        }

        // Record type 0x3B - BeginPath
        pub fn begin_path(&mut self, _size: usize, _value: &[u8]) {
            // No operation needed
        }

        // Record type 0x3C - EndPath
        pub fn end_path(&mut self, _size: usize, _value: &[u8]) {
            // No operation needed
        }

        // Record type 0x3D - CloseFigure
        pub fn close_figure(&mut self, _size: usize, _value: &[u8]) {
            // No operation needed
        }

        // Record type 0x3E - FillPath
        pub fn fill_path(&mut self, size: usize, value: &[u8]) {
            self.rectangle(size, value);
        }

        // Record type 0x3F - StrokeAndFillPath
        pub fn stroke_and_fill_path(&mut self, size: usize, value: &[u8]) {
            self.rectangle(size, value);
        }

        // Record type 0x40 - StrokePath
        pub fn stroke_path(&mut self, size: usize, value: &[u8]) {
            self.rectangle(size, value);
        }

        // Record type 0x41 - FlattenPath
        pub fn flatten_path(&mut self, _size: usize, _value: &[u8]) {
            // No operation needed
        }

        // Record type 0x42 - WidenPath
        pub fn widen_path(&mut self, _size: usize, _value: &[u8]) {
            // No operation needed
        }

        // Record type 0x43 - SelectClipPath
        pub fn select_clip_path(&mut self, _size: usize, value: &[u8]) {
            let mode = Cursor::new(&value[8..12])
                .read_u32::<LittleEndian>()
                .unwrap();
            self.add_iter("RegionMode", mode.to_string(), 8, 4, "<I");
        }

        // Record type 0x44 - AbortPath
        pub fn abort_path(&mut self, _size: usize, _value: &[u8]) {
            // No operation needed
        }

        /// Parses an EMF GDIComment record (Record type 0x46)
        pub fn gdi_comment(&mut self, _size: usize, value: &[u8]) {
            // Extract the 4-byte comment type
            let comment_type = &value[0xC..0x10];
            self.add_iter("Type", format!("{:?}", comment_type), 0xC, 4, "txt");

            // Check for special GDIC comment type
            if comment_type == b"GDIC" {
                let comment_id = Cursor::new(&value[0x10..0x14])
                    .read_u32::<LittleEndian>()
                    .expect("Failed to read comment ID");

                // Look up comment type description
                let comment_desc = GC_IDS.get(&comment_id).map(|s| *s).unwrap_or("unknown");

                self.add_iter(
                    "PubComment ID",
                    format!("{} ({})", comment_id, comment_desc),
                    0x10,
                    4,
                    "<I",
                );

                // Call specialized handler if available
                if let Some(handler) = GCFUNC_IDS.get(&comment_id) {
                    handler(self, 0, value);
                }
            }
        }

        // Record type 0x49 - InvertRgn
        pub fn invert_rgn(&mut self, size: usize, value: &[u8]) {
            self.rectangle(size, value);

            let rds = Cursor::new(&value[0x18..0x1C])
                .read_u32::<LittleEndian>()
                .unwrap();
            self.add_iter("RgnDataSize", rds.to_string(), 0x18, 4, "<I");
            // TODO: Add RegionData->RegionDataHeader parsing
        }

        // Record type 0x55 - Polybezier16
        pub fn polybezier16(&mut self, size: usize, value: &[u8]) {
            self.rectangle(size, value);

            let count = Cursor::new(&value[24..28])
                .read_i32::<LittleEndian>()
                .unwrap();
            self.add_iter("Count", count.to_string(), 24, 4, "<i");

            for i in 0..count {
                self.point_s(value, 28 + (i * 4) as usize, &i.to_string());
            }
        }

        // Record type 0x56 - Polygon16
        pub fn polygon16(&mut self, size: usize, value: &[u8]) {
            self.polybezier16(size, value);
        }

        // Record type 0x57 - Polyline16
        pub fn polyline16(&mut self, size: usize, value: &[u8]) {
            self.polybezier16(size, value);
        }

        // Record type 0x58 - PolybezierTo16
        pub fn polybezier_to16(&mut self, size: usize, value: &[u8]) {
            self.polybezier16(size, value);
        }

        // Record type 0x59 - PolylineTo16
        pub fn polyline_to16(&mut self, size: usize, value: &[u8]) {
            self.polybezier16(size, value);
        }

        // Record type 0x5A - PolyPolyline16
        pub fn poly_polyline16(&mut self, size: usize, value: &[u8]) {
            self.rectangle(size, value);

            let numpoly = Cursor::new(&value[24..28])
                .read_i32::<LittleEndian>()
                .unwrap();
            self.add_iter("NumOfPoly", numpoly.to_string(), 24, 4, "<i");

            let count = Cursor::new(&value[28..32])
                .read_i32::<LittleEndian>()
                .unwrap();
            self.add_iter("Count", count.to_string(), 28, 4, "<i");

            for i in 0..numpoly {
                let offset = 32 + (i * 4) as usize;
                let val = Cursor::new(&value[offset..offset + 4])
                    .read_u32::<LittleEndian>()
                    .unwrap();
                self.add_iter(&format!("PolyPnt {}", i), val.to_string(), offset, 4, "<I");
            }

            for i in 0..count {
                let offset = 32 + (numpoly * 4) as usize + (i * 4) as usize;
                self.point_s(value, offset, &i.to_string());
            }
        }

        // Record type 0x5B - PolyPolygon16
        pub fn poly_polygon16(&mut self, size: usize, value: &[u8]) {
            self.poly_polyline16(size, value);
        }

        // Record type 0x5C - PolyDraw16
        pub fn polydraw16(&mut self, size: usize, value: &[u8]) {
            self.polybezier16(size, value);

            let count = Cursor::new(&value[24..28])
                .read_i32::<LittleEndian>()
                .unwrap();
            self.add_iter("Count", count.to_string(), 28, 4, "<i");

            for i in 0..count {
                let offset = (count * 4 + 28 + i) as usize;
                let ab_type = value[offset];
                self.add_iter(
                    &format!("abType {}", i),
                    ab_type.to_string(),
                    offset,
                    1,
                    "b",
                );
            }
        }

        // Record type 0x5F - ExtCreatePen
        pub fn ext_create_pen(&mut self, _size: usize, value: &[u8]) {
            let obj_id = Cursor::new(&value[8..12])
                .read_u32::<LittleEndian>()
                .unwrap();
            self.add_iter("ObjID", format!("0x{:X}", obj_id), 8, 4, "<I");

            let off_bmi = Cursor::new(&value[12..16])
                .read_u32::<LittleEndian>()
                .unwrap();
            self.add_iter("offBmi", off_bmi.to_string(), 12, 4, "<I");

            let cb_bmi = Cursor::new(&value[16..20])
                .read_u32::<LittleEndian>()
                .unwrap();
            self.add_iter("cbBmi", cb_bmi.to_string(), 16, 4, "<I");

            let off_bits = Cursor::new(&value[20..24])
                .read_u32::<LittleEndian>()
                .unwrap();
            self.add_iter("offBits", off_bits.to_string(), 20, 4, "<I");

            let cb_bits = Cursor::new(&value[24..28])
                .read_u32::<LittleEndian>()
                .unwrap();
            self.add_iter("cbBits", cb_bits.to_string(), 24, 4, "<I");

            let pen_style = Cursor::new(&value[28..32])
                .read_u32::<LittleEndian>()
                .unwrap();
            self.add_iter("PenStyle", pen_style.to_string(), 28, 4, "<I");

            let width = Cursor::new(&value[32..36])
                .read_u32::<LittleEndian>()
                .unwrap();
            self.add_iter("Width", width.to_string(), 32, 4, "<I");

            let brush_style = Cursor::new(&value[36..40])
                .read_u32::<LittleEndian>()
                .unwrap();
            self.add_iter("BrushStyle", brush_style.to_string(), 36, 4, "<I");

            let clr = format!("{:02X}{:02X}{:02X}", value[42], value[41], value[40]);
            self.add_iter("RGB", clr, 40, 3, "clr");

            let brush_hatch = Cursor::new(&value[44..48])
                .read_u32::<LittleEndian>()
                .unwrap();
            self.add_iter("BrushHatch", brush_hatch.to_string(), 44, 4, "<I");

            let num_style = Cursor::new(&value[48..52])
                .read_u32::<LittleEndian>()
                .unwrap();
            self.add_iter("NumEntryStyle", num_style.to_string(), 48, 4, "<I");

            for i in 0..num_style {
                let offset = 52 + (i * 4) as usize;
                let dash_gap: u32 = Cursor::new(&value[offset..offset + 4])
                    .read_u32::<LittleEndian>()
                    .unwrap();

                self.add_iter(
                    &format!("Dash/Gap {}", i),
                    dash_gap.to_string(),
                    offset,
                    4,
                    "<I",
                );
            }

            self.add_iter(
                "BitmapBuffer",
                "(Optional)".to_string(),
                52 + (num_style * 4) as usize,
                0,
                "",
            );
        }

        // Record type 0x62 - SetICMMode
        pub fn set_icm_mode(&mut self, size: usize, value: &[u8]) {
            self.set_bk_mode(size, value);
        }

        // Record type 0x63 - CreateColorSpace
        pub fn create_color_space(&mut self, _size: usize, value: &[u8]) {
            let lh_cs = Cursor::new(&value[8..12])
                .read_u32::<LittleEndian>()
                .unwrap();
            self.add_iter("lhCS", lh_cs.to_string(), 8, 4, "<I");
        }

        // Record type 0x64 - SetColorSpace
        pub fn set_color_space(&mut self, size: usize, value: &[u8]) {
            self.select_object(size, value);
        }

        // Record type 0x65 - DeleteColorSpace
        pub fn delete_color_space(&mut self, size: usize, value: &[u8]) {
            self.select_object(size, value);
        }

        // Record type 0x6D - ForceUFIMapping
        pub fn force_ufi_mapping(&mut self, _size: usize, value: &[u8]) {
            let chk_sum = Cursor::new(&value[8..12])
                .read_u32::<LittleEndian>()
                .unwrap();
            self.add_iter("ChkSum", chk_sum.to_string(), 8, 4, "<I");

            let idx = Cursor::new(&value[12..16])
                .read_u32::<LittleEndian>()
                .unwrap();
            self.add_iter("Idx", idx.to_string(), 12, 4, "<I");
        }

        // Record type 0x70 - SetICMProfileA
        pub fn set_icm_profile_a(&mut self, _size: usize, value: &[u8]) {
            let flags = Cursor::new(&value[8..12])
                .read_u32::<LittleEndian>()
                .unwrap();
            self.add_iter("dwFlags", flags.to_string(), 8, 4, "<I");

            let cb_name = Cursor::new(&value[12..16])
                .read_u32::<LittleEndian>()
                .unwrap();
            self.add_iter("cbName", cb_name.to_string(), 12, 4, "<I");

            let cb_data = Cursor::new(&value[16..20])
                .read_u32::<LittleEndian>()
                .unwrap();
            self.add_iter("cbData", cb_data.to_string(), 16, 4, "<I");

            let name = &value[20..20 + cb_name as usize];
            self.add_iter("Name", format!("{:?}", name), 20, cb_name as usize, "txt");

            self.add_iter(
                "Data",
                "".to_string(),
                20 + cb_name as usize,
                cb_data as usize,
                "txt",
            );
        }

        // Record type 0x71 - SetICMProfileW
        pub fn set_icm_profile_w(&mut self, _size: usize, value: &[u8]) {
            let flags = Cursor::new(&value[8..12])
                .read_u32::<LittleEndian>()
                .unwrap();
            self.add_iter("dwFlags", flags.to_string(), 8, 4, "<I");

            let cb_name = Cursor::new(&value[12..16])
                .read_u32::<LittleEndian>()
                .unwrap();
            self.add_iter("cbName", cb_name.to_string(), 12, 4, "<I");

            let cb_data = Cursor::new(&value[16..20])
                .read_u32::<LittleEndian>()
                .unwrap();
            self.add_iter("cbData", cb_data.to_string(), 16, 4, "<I");

            let name_bytes = &value[20..20 + (cb_name * 2) as usize];
            let (name, _, _) = UTF_16LE.decode(name_bytes);
            self.add_iter("Name", name.to_string(), 20, (cb_name * 2) as usize, "utxt");

            self.add_iter(
                "Data",
                "".to_string(),
                20 + (cb_name * 2) as usize,
                cb_data as usize,
                "txt",
            );
        }

        // Record type 0x73 - SetLayout
        pub fn set_layout(&mut self, _size: usize, value: &[u8]) {
            let mode = Cursor::new(&value[8..12])
                .read_u32::<LittleEndian>()
                .unwrap();
            self.add_iter("LayoutMode", mode.to_string(), 8, 4, "<I");
        }

        // Record type 0x78 - SetTextJustification
        pub fn set_text_justification(&mut self, _size: usize, value: &[u8]) {
            let break_extra = Cursor::new(&value[8..12])
                .read_i32::<LittleEndian>()
                .unwrap();
            self.add_iter("nBreakExtra", break_extra.to_string(), 8, 4, "<i");

            let break_count = Cursor::new(&value[12..16])
                .read_i32::<LittleEndian>()
                .unwrap();
            self.add_iter("nBreakCount", break_count.to_string(), 12, 4, "<i");
        }

        // Record type 0x79 - ClrMatchToTargetW
        pub fn clr_match_to_target_w(&mut self, _size: usize, value: &[u8]) {
            let dw_action = Cursor::new(&value[8..12])
                .read_u32::<LittleEndian>()
                .unwrap();
            let action_desc = COLOR_SPACE
                .iter()
                .find(|&&(id, _)| id == dw_action)
                .map(|&(_, desc)| desc)
                .unwrap_or("unknown");
            self.add_iter(
                "dwAction",
                format!("{} ({})", dw_action, action_desc),
                8,
                4,
                "<I",
            );

            let dw_flags = Cursor::new(&value[12..16])
                .read_u32::<LittleEndian>()
                .unwrap();
            let flags_desc = COLOR_MATCH_TO_TARGET
                .iter()
                .find(|&&(id, _)| id == dw_flags)
                .map(|&(_, desc)| desc)
                .unwrap_or("unknown");
            self.add_iter(
                "dwFlags",
                format!("{} ({})", dw_flags, flags_desc),
                12,
                4,
                "<I",
            );

            let cb_name = Cursor::new(&value[16..20])
                .read_u32::<LittleEndian>()
                .unwrap();
            self.add_iter("cbName", cb_name.to_string(), 16, 4, "<I");

            let cb_data = Cursor::new(&value[20..24])
                .read_u32::<LittleEndian>()
                .unwrap();
            self.add_iter("cbData", cb_data.to_string(), 20, 4, "<I");

            let name_bytes = &value[24..24 + (cb_name * 2) as usize];
            let (name, _, _) = UTF_16LE.decode(name_bytes);
            self.add_iter("Name", name.to_string(), 24, (cb_name * 2) as usize, "utxt");

            self.add_iter(
                "Data",
                "".to_string(),
                24 + (cb_name * 2) as usize,
                cb_data as usize,
                "txt",
            );
        }
    }

    // Helper reading functions
    fn read_i16(data: &[u8]) -> i16 {
        Cursor::new(data).read_i16::<LittleEndian>().unwrap()
    }

    fn read_u16(data: &[u8]) -> u16 {
        Cursor::new(data).read_u16::<LittleEndian>().unwrap()
    }

    fn read_i32(data: &[u8]) -> i32 {
        Cursor::new(data).read_i32::<LittleEndian>().unwrap()
    }

    fn read_u32(data: &[u8]) -> u32 {
        Cursor::new(data).read_u32::<LittleEndian>().unwrap()
    }

    fn read_f32(data: &[u8]) -> f32 {
        Cursor::new(data).read_f32::<LittleEndian>().unwrap()
    }

    // Tree model implementation
    #[derive(Serialize, Debug, Clone)]
    pub struct TreeModel {
        items: Vec<TreeItem>,
    }

    impl TreeModel {
        pub fn new() -> Self {
            TreeModel { items: Vec::new() }
        }

        pub fn add_item(
            &mut self,
            name: &str,
            value: String,
            offset: usize,
            length: usize,
            vtype: &str,
        ) {
            self.items.push(TreeItem {
                name: name.to_string(),
                value,
                offset,
                length,
                vtype: vtype.to_string(),
            });
        }
    }

    #[derive(Debug, Serialize, Clone)]
    struct TreeItem {
        name: String,
        value: String,
        offset: usize,
        length: usize,
        vtype: String,
    }

    // Record type mapping
    lazy_static! {
            static ref GC_IDS: HashMap<u32, &'static str> = {
        let mut m = HashMap::new();
        m.insert(0x80000001, "WindowsMetafile");
        m.insert(2, "BeginGroup");
        m.insert(3, "EndGroup");
        m.insert(0x40000004, "MultiFormats");
        m.insert(0x00000040, "UNICODE_STRING");
        m.insert(0x00000080, "UNICODE_END");
        m
    };

    static ref GCFUNC_IDS: HashMap<u32, fn(&mut EmfParser,size: usize, &[u8])> = {
        let mut m: HashMap<u32, fn(&mut EmfParser,size: usize, &[u8])> = HashMap::new();
        m.insert(2, EmfParser::gc_begin_group);
        m.insert(3, EmfParser::gc_end_group);
        m
    };
        static ref EMR_IDS: HashMap<u32, fn(&mut EmfParser,size: usize, &[u8])> = {
            let mut m: HashMap<u32, fn(&mut EmfParser,size: usize, &[u8])> = HashMap::new();
                // Basic records
        m.insert(0x01, EmfParser::parse_header);
        m.insert(0x02, EmfParser::polybezier);
        m.insert(0x03, EmfParser::polygon);
        m.insert(0x04, EmfParser::polyline);
        m.insert(0x05, EmfParser::polybezier_to);
        m.insert(0x06, EmfParser::polyline_to);
        m.insert(0x07, EmfParser::poly_polyline);
        m.insert(0x08, EmfParser::poly_polygon);

        // Coordinate space and transformation
        m.insert(0x09, EmfParser::set_window_ext_ex);
        m.insert(0x0A, EmfParser::set_window_org_ex);
        m.insert(0x0B, EmfParser::set_viewport_ext_ex);
        m.insert(0x0C, EmfParser::set_viewport_org_ex);
        m.insert(0x0D, EmfParser::set_brush_org_ex);

        // Graphics modes and attributes
        m.insert(0x10, EmfParser::set_mapper_flags);
        m.insert(0x11, EmfParser::set_map_mode);
        m.insert(0x12, EmfParser::set_bk_mode);
        m.insert(0x13, EmfParser::set_polyfill_mode);
        m.insert(0x14, EmfParser::set_rop2);
        m.insert(0x15, EmfParser::set_stretch_blt_mode);
        m.insert(0x16, EmfParser::set_text_align);
        m.insert(0x17, EmfParser::set_color_adjustment);
        m.insert(0x18, EmfParser::set_text_color);
        m.insert(0x19, EmfParser::set_bk_color);

        // Clipping and paths
        m.insert(0x1A, EmfParser::offset_clip_rgn);
        m.insert(0x1B, EmfParser::move_to_ex);
        m.insert(0x1D, EmfParser::exclude_clip_rect);
        m.insert(0x1E, EmfParser::intersect_clip_rect);
        m.insert(0x1F, EmfParser::scale_viewport_ext_ex);
        m.insert(0x20, EmfParser::scale_window_ext_ex);

        // DC state management
        m.insert(0x21, EmfParser::save_dc);
        m.insert(0x22, EmfParser::restore_dc);
        m.insert(0x23, EmfParser::set_world_transform);
        m.insert(0x24, EmfParser::modify_world_transform);

        // Object management
        m.insert(0x25, EmfParser::select_object);
        m.insert(0x26, EmfParser::create_pen);
        m.insert(0x27, EmfParser::create_brush_indirect);
        m.insert(0x28, EmfParser::delete_object);

        // Drawing commands
        m.insert(0x29, EmfParser::angle_arc);
        m.insert(0x2A, EmfParser::ellipse);
        m.insert(0x2B, EmfParser::rectangle);
        m.insert(0x2C, EmfParser::round_rect);
        m.insert(0x2D, EmfParser::arc);
        m.insert(0x2E, EmfParser::chord);
        m.insert(0x2F, EmfParser::pie);
        m.insert(0x30, EmfParser::select_palette);
        m.insert(0x33, EmfParser::resize_palette);
        m.insert(0x35, EmfParser::ext_flood_fill);
        m.insert(0x36, EmfParser::line_to);
        m.insert(0x37, EmfParser::arc_to);
        m.insert(0x38, EmfParser::polydraw);
        m.insert(0x39, EmfParser::set_arc_direction);
        m.insert(0x3A, EmfParser::set_miter_limit);

        // Path operations
        m.insert(0x3B, EmfParser::begin_path);
        m.insert(0x3C, EmfParser::end_path);
        m.insert(0x3D, EmfParser::close_figure);
        m.insert(0x3E, EmfParser::fill_path);
        m.insert(0x3F, EmfParser::stroke_and_fill_path);
        m.insert(0x40, EmfParser::stroke_path);
        m.insert(0x41, EmfParser::flatten_path);
        m.insert(0x42, EmfParser::widen_path);
        m.insert(0x43, EmfParser::select_clip_path);
        m.insert(0x44, EmfParser::abort_path);

        // Special records
        m.insert(0x46, EmfParser::gdi_comment);
        m.insert(0x49, EmfParser::invert_rgn);

        // 16-bit versions
        m.insert(0x55, EmfParser::polybezier16);
        m.insert(0x56, EmfParser::polygon16);
        m.insert(0x57, EmfParser::polyline16);
        m.insert(0x58, EmfParser::polybezier_to16);
        m.insert(0x59, EmfParser::polyline_to16);
        m.insert(0x5A, EmfParser::poly_polyline16);
        m.insert(0x5B, EmfParser::poly_polygon16);
        m.insert(0x5C, EmfParser::polydraw16);

        // Advanced objects
        m.insert(0x5F, EmfParser::ext_create_pen);

        // Color management
        m.insert(0x62, EmfParser::set_icm_mode);
        m.insert(0x63, EmfParser::create_color_space);
        m.insert(0x64, EmfParser::set_color_space);
        m.insert(0x65, EmfParser::delete_color_space);
        m.insert(0x6D, EmfParser::force_ufi_mapping);
        m.insert(0x70, EmfParser::set_icm_profile_a);
        m.insert(0x71, EmfParser::set_icm_profile_w);
        m.insert(0x73, EmfParser::set_layout);
        m.insert(0x78, EmfParser::set_text_justification);
        m.insert(0x79, EmfParser::clr_match_to_target_w);

        m
        };
    }
}
