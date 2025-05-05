# AV1 Decoder Implementation Guide (Rust)

This document was created using an llm with the av1 codecs spec in context (gemini 2.5 pro was used) and will be updated form now on by me a real human person. 
Spec File: [https://aomediacodec.github.io/av1-spec/av1-spec.pdf](https://aomediacodec.github.io/av1-spec/av1-spec.pdf)

**Approach:**

*   **Iterative:** Start with the basics (parsing, I-frames) and incrementally add features (inter-frames, filters, advanced tools).
*   **Spec-Driven:** Refer heavily to the official AV1 specification document. Section numbers are provided below.
*   **Reference:** Utilize the `libaom` reference implementation for comparison and debugging.
*   **Testing:** Test each component thoroughly as it's developed.

---

## Phase 0: Project Setup & Foundation

Goal: Set up the Rust project, basic data structures, and utilities.

*   [x] **Project Initialization:**
    *   Description: Create a new Rust library or binary project using Cargo.
    *   `cargo new av1_decoder --lib` (or `--bin`)
*   [ ] **Bitstream Reader Implementation:**
    *   Description: Create a robust way to read bits from a byte slice or reader. Needs to handle reading single bits, `n` bits, potential byte alignment, and track position. Consider using a crate like `bitstream-io` or implementing a custom one.
    * first try: custom stream implementation (takes a stream and reads bytes and handles bit reads internally via a cursor and a read buffer for byte alignment)
    *   Refs: General programming task.
*   [ ] **Error Handling:**
    *   Description: Define a custom error enum (`DecodeError`?) to handle parsing errors, invalid bitstream states, unsupported features, etc. Use `Result<T, DecodeError>` throughout the parsing logic.
    *   Refs: Rust error handling best practices.
*   [ ] **Core Data Structures:**
    *   Description: Define basic structs for things like `FrameDimensions`, `MotionVector`, `QuantizationParams`, `SegmentationParams`, `LoopFilterParams`, etc. These will be populated during parsing.
    *   Refs: Sections 2, 3, 5 (implicitly).
*   [ ] **Constants Implementation:**
    *   Description: Define constants from the specification (e.g., `MAX_TILE_WIDTH`, `REFS_PER_FRAME`, `BLOCK_SIZES`, `MV_JOINTS`, etc.) in a dedicated module.
    *   Refs: Section 3.
*   [ ] **Mathematical Functions Implementation:**
    *   Description: Implement helper functions defined in the spec.
    *   Refs: Section 4.7.
    *   Tasks:
        *   [ ] `Abs(x)`
        *   [ ] `Clip1(x)` (Clipping based on `BitDepth`)
        *   [ ] `Clip3(min, max, x)`
        *   [ ] `Min(a, b)`, `Max(a, b)`
        *   [ ] `Round2(x, n)` (Integer version)
        *   [ ] `Round2Signed(x, n)`
        *   [ ] `FloorLog2(x)`
        *   [ ] `CeilLog2(x)`

---

## Phase 1: Basic Bitstream Parsing & OBU Handling

Goal: Read the bitstream, identify OBU units, and parse basic data types.

*   [ ] **Syntax Descriptor Parsing (Non-Arithmetic):**
    *   Description: Implement functions within the bitstream reader or a parsing module to handle the basic, non-arithmetically coded descriptors.
    *   Refs: Section 4.10.
    *   Tasks:
        *   [ ] `f(n)` (Read `n` unsigned bits) - Section 4.10.2, 8.1
        *   [ ] `uvlc()` (Unsigned Variable Length Code) - Section 4.10.3
        *   [ ] `le(n)` (Read `n` little-endian bytes) - Section 4.10.4
        *   [ ] `leb128()` (Little-Endian Base 128) - Section 4.10.5
        *   [ ] `su(n)` (Signed `n`-bit integer) - Section 4.10.6
        *   [ ] `ns(n)` (Non-symmetric unsigned integer) - Section 4.10.7
*   [ ] **OBU Header Parsing:**
    *   Description: Read and interpret the OBU header fields.
    *   Refs: Section 5.3.2, 6.2.2.
    *   Tasks:
        *   [ ] Read `obu_forbidden_bit` (and check it's 0).
        *   [ ] Read `obu_type`.
        *   [ ] Read `obu_extension_flag`.
        *   [ ] Read `obu_has_size_field`.
        *   [ ] Read `obu_reserved_1bit` (and check it's 0).
*   [ ] **OBU Extension Header Parsing:**
    *   Description: If `obu_extension_flag` is 1, read the extension header.
    *   Refs: Section 5.3.3, 6.2.3.
    *   Tasks:
        *   [ ] Read `temporal_id`.
        *   [ ] Read `spatial_id`.
        *   [ ] Read `extension_header_reserved_3bits` (and check it's 0).
*   [ ] **OBU Size Parsing:**
    *   Description: If `obu_has_size_field` is 1, read `obu_size` using `leb128()`. Store the size associated with the OBU.
    *   Refs: Section 5.3.1, 4.10.5.
*   [ ] **OBU Iteration/Skipping:**
    *   Description: Implement logic to iterate through OBUs in a bitstream, using `obu_size` (if present) or other framing mechanisms (like Annex B, if supported later) to find the start/end of each OBU. Implement skipping unknown/unsupported OBU types based on their size.
    *   Refs: Section 5.3.1, 6.2.1, Annex B.
*   [ ] **Trailing Bits & Byte Alignment Parsing:**
    *   Description: Implement parsing for `trailing_one_bit` and `trailing_zero_bit` and the `byte_alignment()` function. This is crucial for correctly consuming bits *after* parsing OBU payloads (especially when not using the arithmetic decoder or when skipping).
    *   Refs: Section 5.3.4, 5.3.5, 6.2.4, 6.2.5.

---

## Phase 2: Sequence & Frame Setup Parsing

Goal: Parse sequence-level and frame-level parameters necessary to configure the decoder.

*   [ ] **Sequence Header OBU Parsing:**
    *   Description: Parse the full Sequence Header OBU payload. Store parameters in a persistent decoder context or `SequenceHeader` struct.
    *   Refs: Section 5.5, 6.4.
    *   Tasks:
        *   [ ] Parse `seq_profile`, `still_picture`, `reduced_still_picture_header`.
        *   [ ] Parse timing/model info presence flags.
        *   [ ] Parse operating point information (`operating_points_cnt_minus_1`, `operating_point_idc`, `seq_level_idx`, `seq_tier`).
        *   [ ] Parse display/decoder model info if present (5.5.3, 5.5.4, 5.5.5).
        *   [ ] Parse frame resolution parameters (`frame_width_bits_minus_1`, etc.).
        *   [ ] Parse frame ID parameters if present.
        *   [ ] Parse feature flags (`use_128x128_superblock`, `enable_filter_intra`, etc.).
        *   [ ] Parse Color Configuration (5.5.2, 6.4.2).
        *   [ ] Parse `film_grain_params_present`.
*   [ ] **Frame Header OBU Parsing (General):**
    *   Description: Handle the top-level Frame Header OBU logic, including potentially copying state from a previous header.
    *   Refs: Section 5.9.1, 6.8.1.
*   [ ] **Uncompressed Header Parsing:**
    *   Description: Parse the frame-specific parameters not coded arithmetically. Focus on I-frame needs first. Store in a `FrameHeader` struct or similar.
    *   Refs: Section 5.9.2, 6.8.2.
    *   Tasks:
        *   [ ] Handle `show_existing_frame` logic.
        *   [ ] Parse `frame_type`.
        *   [ ] Parse `show_frame`, `showable_frame`.
        *   [ ] Parse `error_resilient_mode`.
        *   [ ] Parse `disable_cdf_update`.
        *   [ ] Parse `allow_screen_content_tools`.
        *   [ ] Parse `force_integer_mv`.
        *   [ ] Parse `current_frame_id` (if present).
        *   [ ] Parse `frame_size_override_flag`.
        *   [ ] Parse `order_hint`.
        *   [ ] Parse `primary_ref_frame`.
        *   [ ] Parse frame size / render size (5.9.5, 5.9.6).
        *   [ ] Parse `allow_high_precision_mv`.
        *   [ ] Parse `interpolation_filter` (5.9.10).
        *   [ ] Parse `is_motion_mode_switchable`.
        *   [ ] Parse `use_ref_frame_mvs`.
        *   [ ] Parse `disable_frame_end_update_cdf`.
*   [ ] **Tile Info Parsing:**
    *   Description: Parse how the frame is divided into tiles. Calculate `MiCols`, `MiRows`, `TileColsLog2`, `TileRowsLog2`, `MiColStarts`, `MiRowStarts`.
    *   Refs: Section 5.9.15, 6.8.14.

---

## Phase 3: Arithmetic Decoder Implementation

Goal: Implement the core entropy decoding mechanism.

*   [ ] **Symbol Decoder State:**
    *   Description: Implement the core state variables (`SymbolValue`, `SymbolRange`) and the bit reading logic (`read_bit`) considering `SymbolMaxBits`.
    *   Refs: Section 8.2.
*   [ ] **CDF Handling:**
    *   Description: Implement data structures for CDFs. Load default CDFs from Section 9.4. Implement the CDF update logic (`update_cdf`) called by `read_symbol`.
    *   Refs: Section 8.2.6, 8.3, 9.4.
*   [ ] **`read_symbol(cdf)` Implementation:**
    *   Description: Implement the main symbol decoding function using binary search or equivalent over the CDF array to find the symbol, update state, and update the CDF.
    *   Refs: Section 8.2.6.
*   [ ] **`read_bool()` Implementation:**
    *   Description: Implement the boolean decoding process (pseudo-raw bit).
    *   Refs: Section 8.2.3.
*   [ ] **`read_literal(n)` Implementation:**
    *   Description: Implement the function to read an `n`-bit unsigned literal using `read_bool`.
    *   Refs: Section 8.2.5.
*   [ ] **Syntax Descriptor Parsing (Arithmetic):**
    *   Description: Implement functions for arithmetically coded descriptors using the symbol decoder.
    *   Refs: Section 4.10.
    *   Tasks:
        *   [ ] `L(n)` (Literal using `read_literal`) - Section 4.10.8.
        *   [ ] `S()` (Symbol using `read_symbol`) - Section 4.10.9.
        *   [ ] `NS(n)` (Non-symmetric using `L(n)`) - Section 4.10.10.
*   [ ] **Symbol Decoder Init/Exit:**
    *   Description: Implement `init_symbol(sz)` and `exit_symbol()` including reading initial bits and handling/consuming trailing bits.
    *   Refs: Section 8.2.2, 8.2.4.

---

## Phase 4: Intra Frame Decoding Core

Goal: Decode the pixel data for a single Intra frame.

*   [ ] **Tile Group OBU Parsing:**
    *   Description: Parse the Tile Group OBU structure, identifying which tiles are present.
    *   Refs: Section 5.11, 6.10.
*   [ ] **Decode Tile Process:**
    *   Description: Implement the setup for decoding a single tile, including context initialization (`clear_above_context`, `clear_left_context`).
    *   Refs: Section 5.11.2, 6.10.2, 6.10.3.
*   [ ] **Partition Parsing:**
    *   Description: Implement the recursive `decode_partition` function.
    *   Refs: Section 5.11.4, 6.10.4.
*   [ ] **Block Decoding Setup:**
    *   Description: Implement the initial part of `decode_block`, calculating block dimensions (`bw4`, `bh4`), chroma availability (`HasChroma`), and neighbor availability (`AvailU`, `AvailL`, etc.).
    *   Refs: Section 5.11.5, 6.10.5.
*   [ ] **Mode Info Parsing (Intra):**
    *   Description: Implement `intra_frame_mode_info` and related parsing functions (`intra_segment_id`, `read_skip`, `read_skip_mode`, `intra_angle_info_y/uv`, `palette_mode_info`, `filter_intra_mode_info`, `read_cfl_alphas`).
    *   Refs: Section 5.11.7 - 5.11.11, 5.11.22, 5.11.24, 5.11.42, 5.11.43, 5.11.45, 5.11.46.
*   [ ] **Intra Prediction Implementation:**
    *   Description: Implement the actual intra prediction algorithms based on the parsed mode. Start with simpler modes. Requires access to reconstructed neighbor pixels.
    *   Refs: Section 7.11.2 (and sub-sections 7.11.2.1 - 7.11.2.12).
    *   Tasks:
        *   [ ] DC Prediction (7.11.2.5)
        *   [ ] Directional Prediction (7.11.2.4)
        *   [ ] Paeth Prediction (7.11.2.2 - Basic)
        *   [ ] Smooth Prediction (7.11.2.6)
        *   [ ] Filter Intra (7.11.2.3, 7.11.2.7-7.11.2.12)
        *   [ ] Chroma From Luma (CFL) (7.11.5)
        *   [ ] Palette Prediction (7.11.4)
*   [ ] **Transform Block Parsing:**
    *   Description: Implement `read_block_tx_size` / `read_var_tx_size` and `transform_type` parsing.
    *   Refs: Section 5.11.15, 5.11.16, 5.11.17, 5.11.47, 6.10.16 - 6.10.19.
*   [ ] **Coefficient Parsing:**
    *   Description: Implement the `coeffs` syntax parsing, using the arithmetic decoder and appropriate CDF contexts.
    *   Refs: Section 5.11.39, 6.10.34, 8.3.2 (coeff CDFs).
*   [ ] **Dequantization:**
    *   Description: Implement the dequantization functions (`get_dc_quant`, `get_ac_quant`) and apply them to the parsed coefficients. Handle quantizer matrices if `using_qmatrix` is set.
    *   Refs: Section 7.12.2, 6.8.11, 9.5.
*   [ ] **Inverse Transforms:**
    *   Description: Implement the 1D and 2D inverse transforms (IDCT, IADST, Identity, IWHT).
    *   Refs: Section 7.13 (and sub-sections).
*   [ ] **Reconstruction:**
    *   Description: Implement the `reconstruct` process: add the predicted samples and the inverse transformed residual samples. Store the final reconstructed pixels.
    *   Refs: Section 7.12.3.

---

## Phase 5: In-Loop Filters

Goal: Apply post-processing filters to the reconstructed frame.

*   [ ] **Loop Filter Parameter Parsing:**
    *   Description: Parse `loop_filter_params`, `delta_lf_params`.
    *   Refs: Section 5.9.11, 5.9.18, 6.8.10, 6.8.16.
*   [ ] **Deblocking Loop Filter Implementation:**
    *   Description: Implement the edge filtering process based on parsed levels, sharpness, and deltas.
    *   Refs: Section 7.14 (and sub-sections).
*   [ ] **CDEF Parameter Parsing:**
    *   Description: Parse `cdef_params`.
    *   Refs: Section 5.9.19, 6.10.14.
*   [ ] **CDEF Implementation:**
    *   Description: Implement Constrained Directional Enhancement Filter process.
    *   Refs: Section 7.15 (and sub-sections).
*   [ ] **Loop Restoration Parameter Parsing:**
    *   Description: Parse `lr_params`.
    *   Refs: Section 5.9.20, 6.10.15.
*   [ ] **Loop Restoration Implementation:**
    *   Description: Implement Wiener filter and Self-Guided Restoration filter processes.
    *   Refs: Section 7.17 (and sub-sections).

---

## Phase 6: Inter Frame Decoding

Goal: Add support for P-frames and B-frames (frames referencing others).

*   [ ] **Reference Frame Management:**
    *   Description: Implement the `FrameStore` / `BufferPool` concept. Store and retrieve previously decoded frames correctly based on `refresh_frame_flags` and `ref_frame_idx`. Handle reference counting (`DecoderRefCount`).
    *   Refs: Section 6.8.3, 7.8, 7.20, E.2.
*   [ ] **Mode Info Parsing (Inter):**
    *   Description: Implement `inter_frame_mode_info` and related parsing functions (`inter_segment_id`, `read_is_inter`, `inter_block_mode_info`, `read_ref_frames`, `assign_mv`, `read_motion_mode`, `read_interintra_mode`, `read_compound_type`).
    *   Refs: Section 5.11.18 - 5.11.21, 5.11.23, 5.11.25 - 5.11.29.
*   [ ] **Motion Vector Prediction:**
    *   Description: Implement the `find_mv_stack` process and its sub-processes (temporal scan, spatial scan, compound search, sorting, etc.).
    *   Refs: Section 7.10 (and sub-sections).
*   [ ] **Motion Vector Parsing:**
    *   Description: Implement `read_mv` and `read_mv_component` using the arithmetic decoder and MV CDFs.
    *   Refs: Section 5.11.31, 5.11.32, 6.10.29, 6.10.30, 8.3.2 (MV CDFs).
*   [ ] **Inter Prediction / Motion Compensation:**
    *   Description: Implement the `predict_inter` process, including motion vector scaling (7.11.3.3) and block inter prediction (7.11.3.4) using the specified interpolation filters.
    *   Refs: Section 7.11.3.
*   [ ] **Warped Motion:**
    *   Description: Implement warp parameter calculation (7.11.3.8) and warped motion compensation (7.11.3.5).
    *   Refs: Section 7.11.3.5, 7.11.3.8.
*   [ ] **OBMC (Overlapped Block Motion Compensation):**
    *   Description: Implement OBMC blending.
    *   Refs: Section 7.11.3.9.
*   [ ] **Compound Prediction:**
    *   Description: Implement wedge masks (7.11.3.11), difference weighting (7.11.3.12), distance weighting (7.11.3.15), and mask blending (7.11.3.14).
    *   Refs: Section 7.11.3.11 - 7.11.3.15.

---

## Phase 7: Advanced Features & Conformance

Goal: Implement remaining tools and ensure conformance.

*   [ ] **Film Grain:**
    *   Description: Parse film grain parameters (5.9.30, 6.8.20) and implement the synthesis process (7.18.3).
    *   Refs: Section 5.9.30, 6.8.20, 7.18.3.
*   [ ] **Super-Resolution:**
    *   Description: Parse super-res parameters (5.9.8, 6.8.7) and implement the upscaling process (7.16).
    *   Refs: Section 5.9.8, 6.8.7, 7.16.
*   [ ] **Scalability Support:**
    *   Description: Fully handle OBU extension headers and operating points if required.
    *   Refs: Section 6.2.3, 6.4.1, 6.7.5, 6.7.6.
*   [ ] **Error Resilience:**
    *   Description: Implement robust handling for `error_resilient_mode`. Consider strategies from Annex C if targeting non-conformant streams.
    *   Refs: Section 6.8.2, Annex C.
*   [ ] **Decoder Model Conformance:**
    *   Description: Implement checks or a simulation based on Annex E to verify level conformance.
    *   Refs: Annex E.
*   [ ] **Large Scale Tile Decoding (Optional):**
    *   Description: Implement the alternative decoding path using Tile List OBUs.
    *   Refs: Section 5.12, 6.11, 7.3.

---

## Ongoing Tasks

*   [ ] **Unit Testing:** Write tests for individual parsing functions, arithmetic coder components, transform implementations, etc.
*   [ ] **Integration Testing:** Test module interactions (e.g., parsing -> prediction -> reconstruction).
*   [ ] **Stream Testing:** Decode full reference bitstreams (e.g., from `libaom` test vectors) and compare output against the reference decoder or known good checksums/images.
*   [ ] **Refactoring:** Improve code structure, clarity, and maintainability as the implementation grows.
*   [ ] **Performance Optimization:** Profile and optimize critical sections (arithmetic decoding, transforms, motion compensation) *after* correctness is established.
*   [ ] **Documentation:** Add Rustdoc comments and potentially higher-level documentation.

---

This guide provides a structured path. The order can be adjusted slightly, but dependencies exist (e.g., arithmetic coding is needed for most coefficient/MV parsing). Good luck!
