mod impls;
pub const SELECT_SCREEN_CONTENT_TOOLS: u8 = 2u8;
pub const SELECT_INTEGER_MV: u8 = 2u8;


/*
color_primaries	Name of color primaries	Description
1	CP_BT_709	BT.709
2	CP_UNSPECIFIED	Unspecified
4	CP_BT_470_M	BT.470 System M (historical)
5	CP_BT_470_B_G	BT.470 System B, G (historical)
6	CP_BT_601	BT.601
7	CP_SMPTE_240	SMPTE 240
8	CP_GENERIC_FILM	Generic film (color filters using illuminant C)
9	CP_BT_2020	BT.2020, BT.2100
10	CP_XYZ	SMPTE 428 (CIE 1921 XYZ)
11	CP_SMPTE_431	SMPTE RP 431-2
12	CP_SMPTE_432	SMPTE EG 432-1
22	CP_EBU_3213	EBU Tech. 3213-E
*/
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum COLOR_PRIMARIES {
    CP_BT_709 = 1,
    CP_UNSPECIFIED = 2,
    CP_BT_470_M = 4,
    CP_BT_470_B_G = 5,
    CP_BT_601 = 6,
    CP_SMPTE_240 = 7,
    CP_GENERIC_FILM = 8,
    CP_BT_2020 = 9,
    CP_XYZ = 10,
    CP_SMPTE_431 = 11,
    CP_SMPTE_432 = 12,
    CP_EBU_3213 = 22,
}

/*transfer_characteristics is an integer that is defined by the “Transfer characteristics” section of ISO/IEC 23091-4/ITU-T H.273.

transfer_characteristics	Name of transfer characteristics	Description
0	TC_RESERVED_0	For future use
1	TC_BT_709	BT.709
2	TC_UNSPECIFIED	Unspecified
3	TC_RESERVED_3	For future use
4	TC_BT_470_M	BT.470 System M (historical)
5	TC_BT_470_B_G	BT.470 System B, G (historical)
6	TC_BT_601	BT.601
7	TC_SMPTE_240	SMPTE 240 M
8	TC_LINEAR	Linear
9	TC_LOG_100	Logarithmic (100 : 1 range)
10	TC_LOG_100_SQRT10	Logarithmic (100 * Sqrt(10) : 1 range)
11	TC_IEC_61966	IEC 61966-2-4
12	TC_BT_1361	BT.1361
13	TC_SRGB	sRGB or sYCC
14	TC_BT_2020_10_BIT	BT.2020 10-bit systems
15	TC_BT_2020_12_BIT	BT.2020 12-bit systems
16	TC_SMPTE_2084	SMPTE ST 2084, ITU BT.2100 PQ
17	TC_SMPTE_428	SMPTE ST 428
18	TC_HLG	BT.2100 HLG, ARIB STD-B67 */
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TRANSFER_CHARACTERISTICS {
    TC_RESERVED_0 = 0,
    TC_BT_709 = 1,
    TC_UNSPECIFIED = 2,
    TC_RESERVED_3 = 3,
    TC_BT_470_M = 4,
    TC_BT_470_B_G = 5,
    TC_BT_601 = 6,
    TC_SMPTE_240 = 7,
    TC_LINEAR = 8,
    TC_LOG_100 = 9,
    TC_LOG_100_SQRT10 = 10,
    TC_IEC_61966 = 11,
    TC_BT_1361 = 12,
    TC_SRGB = 13,
    TC_BT_2020_10_BIT = 14,
    TC_BT_2020_12_BIT = 15,
    TC_SMPTE_2084 = 16,
    TC_SMPTE_428 = 17,
    TC_HLG = 18,
}


/*matrix_coefficients	Name of matrix coefficients	Description
0	MC_IDENTITY	Identity matrix
1	MC_BT_709	BT.709
2	MC_UNSPECIFIED	Unspecified
3	MC_RESERVED_3	For future use
4	MC_FCC	US FCC 73.628
5	MC_BT_470_B_G	BT.470 System B, G (historical)
6	MC_BT_601	BT.601
7	MC_SMPTE_240	SMPTE 240 M
8	MC_SMPTE_YCGCO	YCgCo
9	MC_BT_2020_NCL	BT.2020 non-constant luminance, BT.2100 YCbCr
10	MC_BT_2020_CL	BT.2020 constant luminance
11	MC_SMPTE_2085	SMPTE ST 2085 YDzDx
12	MC_CHROMAT_NCL	Chromaticity-derived non-constant luminance
13	MC_CHROMAT_CL	Chromaticity-derived constant luminance
14	MC_ICTCP	BT.2100 ICtCp */
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum MATRIX_COEFFICIENTS {
    MC_IDENTITY = 0,
    MC_BT_709 = 1,
    MC_UNSPECIFIED = 2,
    MC_RESERVED_3 = 3,
    MC_FCC = 4,
    MC_BT_470_B_G = 5,
    MC_BT_601 = 6,
    MC_SMPTE_240 = 7,
    MC_SMPTE_YCGCO = 8,
    MC_BT_2020_NCL = 9,
    MC_BT_2020_CL = 10,
    MC_SMPTE_2085 = 11,
    MC_CHROMAT_NCL = 12,
    MC_CHROMAT_CL = 13,
    MC_ICTCP = 14,
}


/*obu_type	Name of obu_type	Layer-specific
0	Reserved	-
1	OBU_SEQUENCE_HEADER.	N
2	OBU_TEMPORAL_DELIMITER	N
3	OBU_FRAME_HEADER	Y
4	OBU_TILE_GROUP	Y
5	OBU_METADATA	See Table in Section 6.7.1
6	OBU_FRAME	Y
7	OBU_REDUNDANT_FRAME_HEADER	Y
8	OBU_TILE_LIST	N
9-14	Reserved	-
15	OBU_PADDING	Either */
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum OBU_TYPE {
    OBU_SEQUENCE_HEADER = 1,
    OBU_TEMPORAL_DELIMITER = 2,
    OBU_FRAME_HEADER = 3,
    OBU_TILE_GROUP = 4,
    OBU_METADATA = 5,
    OBU_FRAME = 6,
    OBU_REDUNDANT_FRAME_HEADER = 7,
    OBU_TILE_LIST = 8,
    OBU_PADDING = 15,
}

/*chroma_sample_position	Name of chroma sample position	Description
0	CSP_UNKNOWN	Unknown (in this case the source video transfer function must be signaled outside the AV1 bitstream)
1	CSP_VERTICAL	Horizontally co-located with (0, 0) luma sample, vertical position in the middle between two luma samples
2	CSP_COLOCATED	co-located with (0, 0) luma sample
3	CSP_RESERVED	 
 */
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CHROMA_SAMPLE_POSITION {
    CSP_UNKNOWN = 0,
    CSP_VERTICAL = 1,
    CSP_COLOCATED = 2,
    CSP_RESERVED = 3,
}
