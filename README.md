# AV1 Decoder Implementation (Rust)

AV1 decoder written in pure rust

Spec File: [https://aomediacodec.github.io/av1-spec/av1-spec.pdf](https://aomediacodec.github.io/av1-spec/av1-spec.pdf)<br>
Impl Spec: [https://aomedia.org/docs/AV1_ToolDescription_v11-clean.pdf](https://aomedia.org/docs/AV1_ToolDescription_v11-clean.pdf)

**Approach:**

*   **(OBU) parsing:** parse bitstream (in progress)
*   **Context for decoding:** keep track of state (in progress)
*   **Decoding implementation:** implement the decoding process (todo)
*   **Post-processing:** apply filters, etc. (todo)
*   **Testing:** write unit tests and collect sample av1 files (in progress)
*   **Output API:** output decoded frames + timing info etc (research needed)

---


## OBU Parsing

    Todos: 
    - [x] Parse OBU header
    - [x] Parse Sequence Header OBU
    - [x] Parse Temporal Delimiter OBU
    - [w] Parse Frame Header OBU
    - [ ] Parse Tile Group OBU
    - [ ] Parse Metadata OBU
    - [ ] Parse Frame OBU
    - [ ] Parse Redundant Frame Header OBU
    - [ ] Parse Tile List OBU
    - [ ] Parse Padding OBU

## Context for decoding

    Todos:
    - [x] Implement context struct
    - [x] Implement reference frame management
    - [todo]

## Decoding implementation

    Todos:
    [todo]

## Post-processing

    Todos:
    [todo]

## Testing

    Todos:
    - [x] Write unit tests for OBU parsing
        - [x] Sequence Header OBU
        - [x] Temporal Delimiter OBU
        - [ ] Frame Header OBU
        - [ ] Tile Group OBU
        - [ ] Metadata OBU
        - [ ] Frame OBU
        - [ ] Redundant Frame Header OBU
        - [ ] Tile List OBU
        - [ ] Padding OBU
    - [ ] Collect sample av1 files
        - [x] Sintel_1080_10s_10MB.mp4
        - [todo]
    - [ ] Desing integration tests

## Output API

    Todos:
    - [ ] Research output API
    - [ ] Decide on output format
    - [ ] Implement output API
    [todo]