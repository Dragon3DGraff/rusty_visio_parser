#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(non_upper_case_globals)]
// src/vsd/constants.rs
/*
 * Константы для работы с документами Visio
 * Соответствует VSDDocumentStructure.h из libvisio
 */

// ================== Типы объектов ==================
/// Идентификаторы типов объектов Visio
pub mod object_types {

    pub const VSD_FOREIGN_DATA: u8 = 0x0c;
    pub const VSD_OLE_LIST: u8 = 0x0d;
    pub const VSD_TEXT: u8 = 0x0e;

    pub const VSD_TRAILER_STREAM: u8 = 0x14;
    pub const VSD_PAGE: u8 = 0x15;
    pub const VSD_COLORS: u8 = 0x16;
    pub const VSD_FONT_LIST: u8 = 0x18;
    pub const VSD_FONT_IX: u8 = 0x19;
    pub const VSD_STYLES: u8 = 0x1a;
    pub const VSD_STENCILS: u8 = 0x1d;
    pub const VSD_STENCIL_PAGE: u8 = 0x1e;
    pub const VSD_OLE_DATA: u8 = 0x1f;

    pub const VSD_PAGES: u8 = 0x27;

    pub const VSD_NAME_LIST: u8 = 0x2c;
    pub const VSD_NAME: u8 = 0x2d;

    pub const VSD_NAME_LIST2: u8 = 0x32;
    pub const VSD_NAME2: u8 = 0x33;
    pub const VSD_NAMEIDX123: u8 = 0x34;

    pub const VSD_PAGE_SHEET: u8 = 0x46;
    pub const VSD_SHAPE_GROUP: u8 = 0x47;
    pub const VSD_SHAPE_SHAPE: u8 = 0x48;
    pub const VSD_SHAPE_GUIDE: u8 = 0x4d;
    pub const VSD_SHAPE_FOREIGN: u8 = 0x4e;

    pub const VSD_STYLE_SHEET: u8 = 0x4a;

    pub const VSD_SCRATCH_LIST: u8 = 0x64;
    pub const VSD_SHAPE_LIST: u8 = 0x65;
    pub const VSD_FIELD_LIST: u8 = 0x66;
    pub const VSD_PROP_LIST: u8 = 0x68;
    pub const VSD_CHAR_LIST: u8 = 0x69;
    pub const VSD_PARA_LIST: u8 = 0x6a;
    pub const VSD_TABS_DATA_LIST: u8 = 0x6b;
    pub const VSD_GEOM_LIST: u8 = 0x6c;
    pub const VSD_CUST_PROPS_LIST: u8 = 0x6d;
    pub const VSD_ACT_ID_LIST: u8 = 0x6e;
    pub const VSD_LAYER_LIST: u8 = 0x6f;
    pub const VSD_CTRL_LIST: u8 = 0x70;
    pub const VSD_C_PNTS_LIST: u8 = 0x71;
    pub const VSD_CONNECT_LIST: u8 = 0x72;
    pub const VSD_HYPER_LNK_LIST: u8 = 0x73;

    pub const VSD_SMART_TAG_LIST: u8 = 0x76;

    pub const VSD_SHAPE_ID: u8 = 0x83;
    pub const VSD_EVENT: u8 = 0x84;
    pub const VSD_LINE: u8 = 0x85;
    pub const VSD_FILL_AND_SHADOW: u8 = 0x86;
    pub const VSD_TEXT_BLOCK: u8 = 0x87;
    pub const VSD_TABS_DATA_1: u8 = 0x88;
    pub const VSD_GEOMETRY: u8 = 0x89;
    pub const VSD_MOVE_TO: u8 = 0x8a;
    pub const VSD_LINE_TO: u8 = 0x8b;
    pub const VSD_ARC_TO: u8 = 0x8c;
    pub const VSD_INFINITE_LINE: u8 = 0x8d;

    pub const VSD_ELLIPSE: u8 = 0x8f;
    pub const VSD_ELLIPTICAL_ARC_TO: u8 = 0x90;

    pub const VSD_PAGE_PROPS: u8 = 0x92;
    pub const VSD_STYLE_PROPS: u8 = 0x93;
    pub const VSD_CHAR_IX: u8 = 0x94;
    pub const VSD_PARA_IX: u8 = 0x95;
    pub const VSD_TABS_DATA_2: u8 = 0x96;
    pub const VSD_TABS_DATA_3: u8 = 0x97;
    pub const VSD_FOREIGN_DATA_TYPE: u8 = 0x98;
    pub const VSD_CONNECTION_POINTS: u8 = 0x99;

    pub const VSD_XFORM_DATA: u8 = 0x9b;
    pub const VSD_TEXT_XFORM: u8 = 0x9c;
    pub const VSD_XFORM_1D: u8 = 0x9d;
    pub const VSD_SCRATCH: u8 = 0x9e;

    pub const VSD_PROTECTION: u8 = 0xa0;
    pub const VSD_TEXT_FIELD: u8 = 0xa1;
    pub const VSD_CONTROL_ANOTHER_TYPE: u8 = 0xa2;

    pub const VSD_MISC: u8 = 0xa4;
    pub const VSD_SPLINE_START: u8 = 0xa5;
    pub const VSD_SPLINE_KNOT: u8 = 0xa6;
    pub const VSD_LAYER_MEMBERSHIP: u8 = 0xa7;
    pub const VSD_LAYER: u8 = 0xa8;
    pub const VSD_ACT_ID: u8 = 0xa9;
    pub const VSD_CONTROL: u8 = 0xaa;

    pub const VSD_USER_DEFINED_CELLS: u8 = 0xb4;
    pub const VSD_TABS_DATA_4: u8 = 0xb5;
    pub const VSD_CUSTOM_PROPS: u8 = 0xb6;
    pub const VSD_RULER_GRID: u8 = 0xb7;

    pub const VSD_CONNECTION_POINTS_ANOTHER_TYPE: u8 = 0xba;

    pub const VSD_DOC_PROPS: u8 = 0xbc;
    pub const VSD_IMAGE: u8 = 0xbd;
    pub const VSD_GROUP: u8 = 0xbe;
    pub const VSD_LAYOUT: u8 = 0xbf;
    pub const VSD_PAGE_LAYOUT_IX: u8 = 0xc0;

    pub const VSD_POLYLINE_TO: u8 = 0xc1;
    pub const VSD_NURBS_TO: u8 = 0xc3;
    pub const VSD_HYPERLINK: u8 = 0xc4;
    pub const VSD_REVIEWER: u8 = 0xc5;
    pub const VSD_ANNOTATION: u8 = 0xc6;
    pub const VSD_SMART_TAG_DEF: u8 = 0xc7;
    pub const VSD_PRINT_PROPS: u8 = 0xc8;
    pub const VSD_NAMEIDX: u8 = 0xc9;

    pub const VSD_SHAPE_DATA: u8 = 0xd1;
    pub const VSD_FONTFACE: u8 = 0xd7;
    pub const VSD_FONTFACES: u8 = 0xd8;
}

// ================== Типы ячеек ==================
/// Типы ячеек (единицы измерения)
pub mod cell_types {
    pub const CELL_TYPE_Number: u8 = 32;
    pub const CELL_TYPE_Percent: u8 = 33;
    pub const CELL_TYPE_Acre: u8 = 36;
    pub const CELL_TYPE_Hectare: u8 = 37;
    pub const CELL_TYPE_Date: u8 = 40;
    pub const CELL_TYPE_DurationUnits: u8 = 42;
    pub const CELL_TYPE_ElapsedWeek: u8 = 43;
    pub const CELL_TYPE_ElapsedDay: u8 = 44;
    pub const CELL_TYPE_ElapsedHour: u8 = 45;
    pub const CELL_TYPE_ElapsedMin: u8 = 46;
    pub const CELL_TYPE_ElapsedSec: u8 = 47;
    pub const CELL_TYPE_TypeUnits: u8 = 48;
    pub const CELL_TYPE_PicasAndPoints: u8 = 49;
    pub const CELL_TYPE_Points: u8 = 50;
    pub const CELL_TYPE_Picas: u8 = 51;
    pub const CELL_TYPE_CicerosAndDidots: u8 = 52;
    pub const CELL_TYPE_Didots: u8 = 53;
    pub const CELL_TYPE_Ciceros: u8 = 54;
    pub const CELL_TYPE_PageUnits: u8 = 63;
    pub const CELL_TYPE_DrawingUnits: u8 = 64;
    pub const CELL_TYPE_Inches: u8 = 65;
    pub const CELL_TYPE_Feet: u8 = 66;
    pub const CELL_TYPE_FeetAndInches: u8 = 67;
    pub const CELL_TYPE_Miles: u8 = 68;
    pub const CELL_TYPE_Centimeters: u8 = 69;
    pub const CELL_TYPE_Millimeters: u8 = 70;
    pub const CELL_TYPE_Meters: u8 = 71;
    pub const CELL_TYPE_Kilometers: u8 = 72;
    pub const CELL_TYPE_InchFractions: u8 = 73;
    pub const CELL_TYPE_MileFractions: u8 = 74;
    pub const CELL_TYPE_Yards: u8 = 75;
    pub const CELL_TYPE_NauticalMiles: u8 = 76;
    pub const CELL_TYPE_AngleUnits: u8 = 80;
    pub const CELL_TYPE_Degrees: u8 = 81;
    pub const CELL_TYPE_DegreeMinuteSecond: u8 = 82;
    pub const CELL_TYPE_Radians: u8 = 83;
    pub const CELL_TYPE_Minutes: u8 = 84;
    pub const CELL_TYPE_Sec: u8 = 85;
    pub const CELL_TYPE_GUID: u8 = 95;
    pub const CELL_TYPE_Currency: u8 = 111;
    pub const CELL_TYPE_NURBS: u8 = 138;
    pub const CELL_TYPE_Polyline: u8 = 139;
    pub const CELL_TYPE_Point: u8 = 225;
    pub const CELL_TYPE_String: u8 = 231;
    pub const CELL_TYPE_StringWithoutUnit: u8 = 232;
    pub const CELL_TYPE_Multidimensional: u8 = 233;
    pub const CELL_TYPE_Color: u8 = 251;
    pub const CELL_TYPE_NoCast: u8 = 252;
    pub const CELL_TYPE_Invalid: u8 = 255;
}

// ================== Форматы полей ==================
/// Форматы полей документа Visio
pub mod field_formats {
    pub const VSD_FIELD_FORMAT_NumGenNoUnits: u16 = 0;
    pub const VSD_FIELD_FORMAT_NumGenDefUnits: u16 = 1;
    pub const VSD_FIELD_FORMAT_0PlNoUnits: u16 = 2;
    pub const VSD_FIELD_FORMAT_0PlDefUnits: u16 = 3;
    pub const VSD_FIELD_FORMAT_1PlNoUnits: u16 = 4;
    pub const VSD_FIELD_FORMAT_1PlDefUnits: u16 = 5;
    pub const VSD_FIELD_FORMAT_2PlNoUnits: u16 = 6;
    pub const VSD_FIELD_FORMAT_2PlDefUnits: u16 = 7;
    pub const VSD_FIELD_FORMAT_3PlNoUnits: u16 = 8;
    pub const VSD_FIELD_FORMAT_3PlDefUnits: u16 = 9;
    pub const VSD_FIELD_FORMAT_FeetAndInches: u16 = 10;
    pub const VSD_FIELD_FORMAT_Radians: u16 = 11;
    pub const VSD_FIELD_FORMAT_Degrees: u16 = 12;
    pub const VSD_FIELD_FORMAT_FeetAndInches1Pl: u16 = 13;
    pub const VSD_FIELD_FORMAT_FeetAndInches2Pl: u16 = 14;
    pub const VSD_FIELD_FORMAT_Fraction1PlNoUnits: u16 = 15;
    pub const VSD_FIELD_FORMAT_Fraction1PlDefUnits: u16 = 16;
    pub const VSD_FIELD_FORMAT_Fraction2PlNoUnits: u16 = 17;
    pub const VSD_FIELD_FORMAT_Fraction2PlDefUnits: u16 = 18;

    pub const VSD_FIELD_FORMAT_DateShort: u16 = 20;
    pub const VSD_FIELD_FORMAT_DateLong: u16 = 21;
    pub const VSD_FIELD_FORMAT_DateMDYY: u16 = 22;
    pub const VSD_FIELD_FORMAT_DateMMDDYY: u16 = 23;
    pub const VSD_FIELD_FORMAT_DateMMMDYYYY: u16 = 24;
    pub const VSD_FIELD_FORMAT_DateMMMMDYYYY: u16 = 25;
    pub const VSD_FIELD_FORMAT_DateDMYY: u16 = 26;
    pub const VSD_FIELD_FORMAT_DateDDMMYY: u16 = 27;
    pub const VSD_FIELD_FORMAT_DateDMMMYYYY: u16 = 28;
    pub const VSD_FIELD_FORMAT_DateDMMMMYYYY: u16 = 29;
    pub const VSD_FIELD_FORMAT_TimeGen: u16 = 30;
    pub const VSD_FIELD_FORMAT_TimeHMM: u16 = 31;
    pub const VSD_FIELD_FORMAT_TimeHHMM: u16 = 32;
    pub const VSD_FIELD_FORMAT_TimeHMM24: u16 = 33;
    pub const VSD_FIELD_FORMAT_TimeHHMM24: u16 = 34;
    pub const VSD_FIELD_FORMAT_TimeHMMAMPM: u16 = 35;
    pub const VSD_FIELD_FORMAT_TimeHHMMAMPM: u16 = 36;
    pub const VSD_FIELD_FORMAT_StrNormal: u16 = 37;
    pub const VSD_FIELD_FORMAT_StrLower: u16 = 38;
    pub const VSD_FIELD_FORMAT_StrUpper: u16 = 39;

    pub const VSD_FIELD_FORMAT_Dateyyyymd: u16 = 44;
    pub const VSD_FIELD_FORMAT_Dateyymmdd: u16 = 45;
    pub const VSD_FIELD_FORMAT_TimeAMPMhmm_J: u16 = 46;

    pub const VSD_FIELD_FORMAT_DateTWNfYYYYMMDDD_C: u16 = 50;
    pub const VSD_FIELD_FORMAT_DateTWNsYYYYMMDDD_C: u16 = 51;
    pub const VSD_FIELD_FORMAT_DateTWNfyyyymmddww_C: u16 = 52;
    pub const VSD_FIELD_FORMAT_DateTWNfyyyymmdd_C: u16 = 53;
    pub const VSD_FIELD_FORMAT_Dategggemdww_J: u16 = 54;
    pub const VSD_FIELD_FORMAT_Dateyyyymdww_J: u16 = 55;
    pub const VSD_FIELD_FORMAT_Dategggemd_J: u16 = 56;
    pub const VSD_FIELD_FORMAT_Dateyyyymd_J: u16 = 57;
    pub const VSD_FIELD_FORMAT_DateYYYYMMMDDDWWW_C: u16 = 58;
    pub const VSD_FIELD_FORMAT_DateYYYYMMMDDD_C: u16 = 59;
    pub const VSD_FIELD_FORMAT_DategeMMMMddddww_K: u16 = 60;
    pub const VSD_FIELD_FORMAT_Dateyyyymdww_K: u16 = 61;
    pub const VSD_FIELD_FORMAT_DategeMMMMddd_K: u16 = 62;
    pub const VSD_FIELD_FORMAT_Dateyyyymd_K: u16 = 63;
    pub const VSD_FIELD_FORMAT_Dateyyyy_m_d: u16 = 64;
    pub const VSD_FIELD_FORMAT_Dateyy_mm_dd: u16 = 65;
    pub const VSD_FIELD_FORMAT_TimeAMPMhmm_C: u16 = 66;
    pub const VSD_FIELD_FORMAT_TimeAMPMhmm_K: u16 = 67;
    pub const VSD_FIELD_FORMAT_TimeAMPM_hmm_J: u16 = 68;
    pub const VSD_FIELD_FORMAT_Timehmm_J: u16 = 69;
    pub const VSD_FIELD_FORMAT_TimeAMPM_hmm_C: u16 = 70;
    pub const VSD_FIELD_FORMAT_Timehmm_C: u16 = 71;
    pub const VSD_FIELD_FORMAT_TimeAMPM_hmm_K: u16 = 72;
    pub const VSD_FIELD_FORMAT_Timehmm_K: u16 = 73;
    pub const VSD_FIELD_FORMAT_TimeHMMAMPM_E: u16 = 74;
    pub const VSD_FIELD_FORMAT_TimeHHMMAMPM_E: u16 = 75;
    pub const VSD_FIELD_FORMAT_Dateyyyymd_S: u16 = 76;
    pub const VSD_FIELD_FORMAT_Dateyyyymmdd_S: u16 = 77;
    pub const VSD_FIELD_FORMAT_Datewwyyyymmdd_S: u16 = 78;
    pub const VSD_FIELD_FORMAT_Datewwyyyymd_S: u16 = 79;
    pub const VSD_FIELD_FORMAT_TimeAMPMhmm_S: u16 = 80;
    pub const VSD_FIELD_FORMAT_TimeAMPMhhmm_S: u16 = 81;

    pub const VSD_FIELD_FORMAT_MsoDateShort: u16 = 200;
    pub const VSD_FIELD_FORMAT_MsoDateLongDay: u16 = 201;
    pub const VSD_FIELD_FORMAT_MsoDateLong: u16 = 202;
    pub const VSD_FIELD_FORMAT_MsoDateShortAlt: u16 = 203;
    pub const VSD_FIELD_FORMAT_MsoDateISO: u16 = 204;
    pub const VSD_FIELD_FORMAT_MsoDateShortMon: u16 = 205;
    pub const VSD_FIELD_FORMAT_MsoDateShortSlash: u16 = 206;
    pub const VSD_FIELD_FORMAT_MsoDateShortAbb: u16 = 207;
    pub const VSD_FIELD_FORMAT_MsoDateEnglish: u16 = 208;
    pub const VSD_FIELD_FORMAT_MsoDateMonthYr: u16 = 209;
    pub const VSD_FIELD_FORMAT_MsoDateMon_Yr: u16 = 210;
    pub const VSD_FIELD_FORMAT_MsoTimeDatePM: u16 = 211;
    pub const VSD_FIELD_FORMAT_MsoTimeDateSecPM: u16 = 212;
    pub const VSD_FIELD_FORMAT_MsoTimePM: u16 = 213;
    pub const VSD_FIELD_FORMAT_MsoTimeSecPM: u16 = 214;
    pub const VSD_FIELD_FORMAT_MsoTime24: u16 = 215;
    pub const VSD_FIELD_FORMAT_MsoTimeSec24: u16 = 216;
    pub const VSD_FIELD_FORMAT_MsoFEExtra1: u16 = 217;
    pub const VSD_FIELD_FORMAT_MsoFEExtra2: u16 = 218;
    pub const VSD_FIELD_FORMAT_MsoFEExtra3: u16 = 219;
    pub const VSD_FIELD_FORMAT_MsoFEExtra4: u16 = 220;
    pub const VSD_FIELD_FORMAT_MsoFEExtra5: u16 = 221;

    pub const VSD_FIELD_FORMAT_Unknown: u16 = 0xffff;
}
