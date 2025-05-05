use crate::VSDInternalStream::VSDInternalStream;

use crate::vsd_constants::object_types::{
    VSD_FONTFACES, VSD_NAME_LIST2, VSD_NAMEIDX, VSD_NAMEIDX123, VSD_OLE_LIST, VSD_PAGE,
    VSD_SHAPE_GROUP, VSD_SHAPE_SHAPE, VSD_STENCIL_PAGE, VSD_STENCILS, VSD_TRAILER_STREAM,
};
use byteorder::{LittleEndian, ReadBytesExt};
use std::collections::{HashMap, HashSet};

use std::io::Cursor;
use std::io::{Read, Seek, SeekFrom};

pub struct VSDParser {
    input: Cursor<Vec<u8>>,
}

impl VSDParser {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            input: Cursor::new(data),
        }
    }
    pub fn parse_main(&mut self) -> bool {
        // if self.input.is_none() {
        //     return false;
        // }

        // Seek to trailer stream pointer
        if let Err(_) = self.input.seek(SeekFrom::Start(0x24)) {
            return false;
        }

        // let trailer_pointer = match self.read_pointer(self.input) {
        //     Ok(ptr) => ptr,
        //     Err(_) => return false,
        // };

        let mut trailer_pointer = Pointer::new();

        let _ = trailer_pointer.read(&mut self.input);

        let compressed = (trailer_pointer.format & 2) == 2;
        let shift = if compressed { 4 } else { 0 };

        if let Err(_) = self
            .input
            .seek(SeekFrom::Start(trailer_pointer.offset as u64))
        {
            return false;
        }

        let mut trailer_stream = VSDInternalStream::new(
            &mut self.input,
            trailer_pointer.length.try_into().unwrap(),
            compressed,
        )
        .unwrap();

        let mut group_xforms_sequence: Vec<HashMap<u32, XForm>> = Vec::new();
        let mut group_memberships_sequence: Vec<HashMap<u32, u32>> = Vec::new();
        let mut document_page_shape_orders: Vec<Vec<u32>> = Vec::new();

        // First pass - styles collection
        // let mut styles_collector = VSDStylesCollector::new(
        //     &mut group_xforms_sequence,
        //     &mut group_memberships_sequence,
        //     &mut document_page_shape_orders,
        // );
        // self.collector = Some(Box::new(styles_collector));

        println!("VSDParser::parse_main 1st pass");
        if !self.parse_document(&mut trailer_stream, shift) {
            return false;
        }

        // self.handle_level_change(0);

        // let styles = styles_collector.get_style_sheets();

        // // Second pass - content collection
        // let content_collector = VSDContentCollector::new(
        //     self.painter,
        //     group_xforms_sequence,
        //     group_memberships_sequence,
        //     document_page_shape_orders,
        //     styles,
        //     &self.stencils,
        // );
        // self.collector = Some(Box::new(content_collector));

        // if self.container.is_some() {
        //     self.parse_meta_data();
        // }

        println!("VSDParser::parse_main 2nd pass");
        if !self.parse_document(&mut trailer_stream, shift) {
            return false;
        }

        true
    }

    fn read_pointer_info<R: Read + Seek>(
        &mut self,
        input: &mut R,
        shift: u32,
        list_size: &mut u32,
        pointer_count: &mut u32,
    ) -> Result<(u32, u32), Box<dyn std::error::Error>> {
        println!(
            "{} {}",
            "VSDParser::readPointerInfo",
            input.stream_position().unwrap()
        );
        println!("shift {}", shift);
        // Переходим к позиции shift
        let _ = input.seek(SeekFrom::Start(shift as u64))?;

        // Читаем смещение
        let offset = input.read_u32::<LittleEndian>()?;

        println!("offset {}", offset);

        // Переходим к позиции offset + shift - 4
        let new_pos = (offset as u64) + (shift as u64) - 4;

        println!("new_pos {}", new_pos);
        let _ = input.seek(SeekFrom::Start(new_pos))?;

        // Читаем размер списка и количество указателей
        let mut list_size_val = match input.read_u32::<LittleEndian>() {
            Ok(va) => va,
            Err(_) => 0,
        };
        println!("list_size {}", list_size);

        let pointer_count_val = match input.read_u32::<LittleEndian>() {
            Ok(va) => va,
            Err(_) => 0,
        };

        *list_size = list_size_val;
        *pointer_count = pointer_count_val;

        // Пропускаем 4 байта
        let _ = input.seek(SeekFrom::Current(4));

        Ok((list_size_val, pointer_count_val))
    }

    pub fn parse_document(&mut self, input: &mut VSDInternalStream, shift: u32) -> bool {
        let mut visited = HashSet::new();
        match self.handle_streams(input, VSD_TRAILER_STREAM.into(), shift, 0, &mut visited) {
            Ok(_) => {
                assert!(visited.is_empty());
                true
            }
            Err(_) => {
                assert!(visited.is_empty());
                false
            }
        }
    }

    fn handle_streams(
        &mut self,
        input: &mut VSDInternalStream,
        ptr_type: u32,
        shift: u32,
        level: u32,
        visited: &mut HashSet<u32>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("VSDParser::handle_streams");

        let mut pointer_order = Vec::new();
        let mut ptr_list = HashMap::new();
        let mut font_faces = HashMap::new();
        let mut name_list = HashMap::new();
        let mut name_idx = HashMap::new();

        // Parse out pointers to streams
        let mut list_size = 0;
        let mut pointer_count = 0;
        self.read_pointer_info(input, shift, &mut list_size, &mut pointer_count)?;

        let mut input_cursor: Cursor<Vec<u8>> = Cursor::new(input.buffer.to_owned());

        for i in 0..pointer_count {
            let mut ptr = Pointer::new();
            let _ = ptr.read(&mut input_cursor);
            if ptr.type_name == 0 {
                continue;
            }

            match ptr.type_name as u8 {
                VSD_FONTFACES => {
                    println!("font_faces.insert");
                    font_faces.insert(i as u32, ptr);
                }
                VSD_NAME_LIST2 => {
                    println!("name_list.insert");
                    name_list.insert(i as u32, ptr);
                }
                VSD_NAMEIDX | VSD_NAMEIDX123 => {
                    println!("name_idx.insert");
                    name_idx.insert(i as u32, ptr);
                }
                _ => {
                    println!("ptr_list.insert");
                    ptr_list.insert(i as u32, ptr);
                }
            }
        }

        if list_size <= 1 {
            list_size = 0;
        }

        for _ in 0..list_size {
            pointer_order.push(input.read_u32::<byteorder::LittleEndian>());
        }

        // Process the streams in specific order
        for (idx, ptr) in name_list {
            self.handle_stream(ptr, idx, level + 1, visited)?;
        }

        for (idx, ptr) in name_idx {
            self.handle_stream(ptr, idx, level + 1, visited)?;
        }

        for (idx, ptr) in font_faces {
            self.handle_stream(ptr, idx, level + 1, visited)?;
        }

        if !pointer_order.is_empty() {
            for j in pointer_order {
                let j_v = &j.unwrap();
                // println!("j_v {}", j_v);
                if let Some(ptr) = ptr_list.remove(&j_v) {
                    self.handle_stream(ptr, *j_v, level + 1, visited)?;
                }
            }
        }

        for (idx, ptr) in ptr_list {
            self.handle_stream(ptr, idx, level + 1, visited)?;
        }

        Ok(())
    }

    fn handle_stream(
        &mut self,
        ptr: Pointer,
        idx: u32,
        level: u32,
        visited: &mut HashSet<u32>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "VSDParser::handle_stream {} type 0x{:x}",
            idx, ptr.type_name
        );

        // self.header.level = level;
        // self.header.id = idx;
        // self.header.chunk_type = ptr.type;
        // self.handle_level_change(level);

        // let mut tmp_stencil = VSDStencil::new();
        let compressed = (ptr.format & 2) == 2;

        println!("compressed {}", compressed);

        self.input.seek(SeekFrom::Start(ptr.offset as u64))?;
        // let mut tmp_input = VSDInternalStream::new(
        //     self.input,
        //     ptr.length,
        //     compressed
        // );

        // self.header.data_length = tmp_input.get_size();
        let shift = if compressed { 4 } else { 0 };

        // // Handle different stream types
        match ptr.type_name as u8 {
            // VSD_STYLES => self.is_in_styles = true,
            // VSD_PAGES => if self.extract_stencils { return Ok(()); },
            VSD_PAGE => {
                print!("Страница")
                // if self.extract_stencils { return Ok(()); }
                // self.is_background_page = (ptr.format & 0x1) == 0;
                // self.name_from_id(&mut self.current_page_name, idx, level + 1);
                // self.collector.as_mut().unwrap().start_page(idx);
            }
            VSD_STENCILS => {
                print!("VSD_STENCILS")
                // if self.extract_stencils { return Ok(()); }
                // if self.stencils.count() > 0 { return Ok(()); }
                // self.is_stencil_started = true;
            }
            VSD_STENCIL_PAGE => {
                print!("VSD_STENCIL_PAGE")
                // if self.extract_stencils {
                //     self.is_background_page = false;
                //     self.name_from_id(&mut self.current_page_name, idx, level + 1);
                //     self.collector.as_mut().unwrap().start_page(idx);
                // } else {
                //     self.current_stencil = Some(&mut tmp_stencil);
                // }
            }
            // VSD_SHAPE_GROUP | VSD_SHAPE_SHAPE | VSD_SHAPE_FOREIGN => {
            //     print!("VSD_SHAPE_GROUP | VSD_SHAPE_SHAPE | VSD_SHAPE_FOREIGN")
            //     // self.current_shape_id = idx;
            // },
            VSD_OLE_LIST => {
                print!("VSD_OLE_LIST")
                // if self.shape.foreign.is_none() {
                //     self.shape.foreign = Some(ForeignData::new());
                // }
                // self.shape.foreign.as_mut().unwrap().data_id = idx;
            }
            _ => {}
        }

        // // Process the stream content
        // match ptr.format >> 4 {
        //     0x4 | 0x5 | 0x0 => {
        //         self.handle_blob(&mut tmp_input, shift, level + 1)?;

        //         if (ptr.format >> 4) == 0x5 && ptr.type != VSD_COLORS {
        //             if visited.insert(ptr.offset) {
        //                 match self.handle_streams(&tmp_input, ptr.type, shift, level + 1, visited) {
        //                     Ok(_) => { visited.remove(&ptr.offset); },
        //                     Err(e) => {
        //                         visited.remove(&ptr.offset);
        //                         return Err(e);
        //                     }
        //                 }
        //             }
        //         }
        //     },
        //     0xD | 0xC | 0x8 => {
        //         self.handle_chunks(&mut tmp_input, level + 1)?;
        //     },
        //     _ => {}
        // }

        // // Clean up after processing
        // match ptr.type {
        //     VSD_STYLES => {
        //         self.handle_level_change(0);
        //         self.is_in_styles = false;
        //     },
        //     VSD_PAGE => {
        //         self.handle_level_change(0);
        //         self.collector.as_mut().unwrap().end_page();
        //     },
        //     VSD_PAGES => {
        //         self.handle_level_change(0);
        //         self.collector.as_mut().unwrap().end_pages();
        //     },
        //     VSD_STENCILS => {
        //         self.handle_level_change(0);
        //         if self.extract_stencils {
        //             self.collector.as_mut().unwrap().end_pages();
        //         } else {
        //             self.is_stencil_started = false;
        //         }
        //     },
        //     VSD_STENCIL_PAGE => {
        //         self.handle_level_change(0);
        //         if self.extract_stencils {
        //             self.collector.as_mut().unwrap().end_page();
        //         } else if let Some(stencil) = self.current_stencil.take() {
        //             self.stencils.add_stencil(idx, stencil);
        //         }
        //     },
        //     VSD_SHAPE_GROUP | VSD_SHAPE_SHAPE | VSD_SHAPE_FOREIGN => {
        //         if self.is_stencil_started {
        //             self.handle_level_change(0);
        //             if let Some(stencil) = &mut self.current_stencil {
        //                 stencil.add_stencil_shape(self.shape.shape_id, self.shape.clone());
        //             }
        //         }
        //     },
        //     _ => {}
        // }

        Ok(())
    }
}

#[derive(Debug, Clone)]
struct Pointer {
    pub type_name: u32,
    pub format: u32,
    pub offset: u32,
    pub length: u32,
    // pub list_size: u16,
}

impl Pointer {
    fn new() -> Self {
        Self {
            type_name: 0,
            format: 0,
            offset: 0,
            length: 0,
            // list_size: 0,
        }
    }
    fn read(&mut self, stream: &mut Cursor<Vec<u8>>) -> Result<(), Box<dyn std::error::Error>> {
        self.type_name = stream.read_u32::<byteorder::LittleEndian>()?;
        let _ = stream.seek(SeekFrom::Current(4))?;
        self.offset = stream.read_u32::<byteorder::LittleEndian>()?;
        self.length = stream.read_u32::<byteorder::LittleEndian>()?;
        self.format = stream.read_u32::<byteorder::LittleEndian>()?;
        // self.list_size = stream.read_u16::<byteorder::LittleEndian>()?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct XForm {
    pub pin_x: f64,
    pub pin_y: f64,
    pub height: f64,
    pub width: f64,
    pub pin_loc_x: f64,
    pub pin_loc_y: f64,
    pub angle: f64,
    pub flip_x: bool,
    pub flip_y: bool,
    pub x: f64,
    pub y: f64,
}

impl XForm {
    pub fn new() -> Self {
        Self::default()
    }
}
