# Third-party notices

The NUS-H2 engine is MIT licensed (see `LICENSE`). The engine, and the WebAssembly module
`nush2.wasm` that the site serves, compile in the Rust crates below, each under its own licence.
Where a crate offers a choice, the licence named as elected is the one this project uses, and its
text follows. Versions are the ones `Cargo.lock` pins.

| Crate | Version | Offered | Elected |
|---|---|---|---|
| itoa | 1.0.18 | MIT OR Apache-2.0 | MIT |
| memchr | 2.8.3 | Unlicense OR MIT | Unlicense |
| serde | 1.0.229 | MIT OR Apache-2.0 | MIT |
| serde_core | 1.0.229 | MIT OR Apache-2.0 | MIT |
| serde_json | 1.0.154 | MIT OR Apache-2.0 | MIT |
| seuif97 | 2.3.8 | MIT | MIT |
| zmij | 1.0.23 | MIT | MIT |

Build-time crates, which are the procedural macro serde_derive and the crates it uses; they run while the engine compiles, and none of their own code is in the module: serde_derive 1.0.229 (MIT OR Apache-2.0), proc-macro2 1.0.107 (MIT OR Apache-2.0), quote 1.0.47 (MIT OR Apache-2.0), syn 3.0.9 (MIT OR Apache-2.0), unicode-ident 1.0.27 ((MIT OR Apache-2.0) AND Unicode-3.0).

Figures and photographs in the report and the presentation that are credited to other sources,
and any other third-party image in them, belong to their owners and are not covered by the MIT
licence; see `README.md`, section "Licence and credit".

## Third-party images in the report and the presentation

The images below sit in `presentation/NUSH2_Final_Presentation.pptx` and its PDF, and in
`report/NUSH2_Final_Report.docx` and its PDF. They belong to their owners, the MIT licence does not
cover them, and this repository grants no right to reuse them. The credit column gives the source
as the file prints it: on the image itself, in the figure caption, in the slide's source line or in
the picture's alternative text. "None printed" means the file carries no credit for that image.
Slide numbers are pages of the presentation PDF.

| Where | What it shows | Source as printed |
|---|---|---|
| Slides 1 and 4 | Chimney with a plume of smoke, a photograph in purple duotone | None printed |
| Slide 2 | Industrial skyline with chimneys, a photograph in purple duotone | None printed |
| Slide 3 | Close view of industrial stacks, a photograph in purple duotone | None printed |
| Slide 13 | Diagram of a neutron splitting a uranium-235 nucleus | Source line: Nuclear-Power.com |
| Slide 24 | Map graphic of the planned new western island and its link to Jurong Island | On the image: Prime Minister's Office; photo, Google Earth; Straits Times Graphics |
| Slide 35 | Render of the Xe-100 reactor vessel and steam generator, shown as an analogue only | Source line: X-energy |
| Slide 57 | Photographs of the ACP100, KLT-40S, NuScale and BWRX-300 designs | None printed; the alternative text names each design |
| Slides 58 and 59 | Cutaway of a TRISO coated particle | Alternative text: INL |
| Slides 58 and 61 | Drawing of a pebble-bed HTGR reactor vessel (HTR-PM) | None printed |
| Slide 62 | Frequency-consequence target for emergency planning, drawn on the slide with the dose in rem | Caption: slightly modified from the Preliminary Safety Analysis Report submitted to the US NRC for the 4 x 80 MWe Xe-100 at Long Mott Generating Station, Texas |
| Slide 65 | Rendering of the Xuwei site | None printed |
| Slide 67 | Spent fuel pool, dry cask storage and an illustration of a deep geological repository | None printed |
| Slide 70 | Photographs of Big Rock Point, Diablo Canyon, Kashiwazaki and Olkiluoto 3, and an image of a fusion device | Captions name the plants; no source printed |
| Slide 71 | Map of Singapore | None printed |
| Report, Figure 1 | Process-heat temperature ranges of industrial applications, with the ranges that light-water reactors and HTGRs serve; the image carries the identifier 08-GA50599-04 | Caption: D. Mears, Module 2b: HTGR Motivations and Applications, NRC HTGR Technology Course, 2010 (slide: Process Application Temperatures) |

Two further figures were drawn for this project after a cited figure. They hold no third-party
pixels, and the figures they follow keep their owners' rights:

| Where | What it shows | Drawn after |
|---|---|---|
| Slide 12 | Process-heat temperature ranges, redrawn as a bar chart | D. Mears, 2010, as the slide's source line states |
| Slide 21 and report Figure A1 | Frequency-consequence target behind an emergency planning zone, a schematic with no values | The Xe-100 plume exposure pathway EPZ report in the Long Mott Generating Station construction permit application, NRC ADAMS ML25090A066 |

Every other image in the four files is the project's own: the NUS-H2 process flow figure and its
crops, the roadmap, the equation image, the crossed-out conventional plant drawings on slide 17,
and the charts drawn from `results/` through `build/numbers.json`.

## itoa 1.0.18, MIT

```text
Permission is hereby granted, free of charge, to any
person obtaining a copy of this software and associated
documentation files (the "Software"), to deal in the
Software without restriction, including without
limitation the rights to use, copy, modify, merge,
publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software
is furnished to do so, subject to the following
conditions:

The above copyright notice and this permission notice
shall be included in all copies or substantial portions
of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
DEALINGS IN THE SOFTWARE.
```

## memchr 2.8.3, Unlicense

```text
This is free and unencumbered software released into the public domain.

Anyone is free to copy, modify, publish, use, compile, sell, or
distribute this software, either in source code form or as a compiled
binary, for any purpose, commercial or non-commercial, and by any
means.

In jurisdictions that recognize copyright laws, the author or authors
of this software dedicate any and all copyright interest in the
software to the public domain. We make this dedication for the benefit
of the public at large and to the detriment of our heirs and
successors. We intend this dedication to be an overt act of
relinquishment in perpetuity of all present and future rights to this
software under copyright law.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY CLAIM, DAMAGES OR
OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE,
ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
OTHER DEALINGS IN THE SOFTWARE.

For more information, please refer to <http://unlicense.org/>
```

## serde 1.0.229, MIT

```text
Permission is hereby granted, free of charge, to any
person obtaining a copy of this software and associated
documentation files (the "Software"), to deal in the
Software without restriction, including without
limitation the rights to use, copy, modify, merge,
publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software
is furnished to do so, subject to the following
conditions:

The above copyright notice and this permission notice
shall be included in all copies or substantial portions
of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
DEALINGS IN THE SOFTWARE.
```

## serde_core 1.0.229, MIT

```text
Permission is hereby granted, free of charge, to any
person obtaining a copy of this software and associated
documentation files (the "Software"), to deal in the
Software without restriction, including without
limitation the rights to use, copy, modify, merge,
publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software
is furnished to do so, subject to the following
conditions:

The above copyright notice and this permission notice
shall be included in all copies or substantial portions
of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
DEALINGS IN THE SOFTWARE.
```

## serde_json 1.0.154, MIT

```text
Permission is hereby granted, free of charge, to any
person obtaining a copy of this software and associated
documentation files (the "Software"), to deal in the
Software without restriction, including without
limitation the rights to use, copy, modify, merge,
publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software
is furnished to do so, subject to the following
conditions:

The above copyright notice and this permission notice
shall be included in all copies or substantial portions
of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
DEALINGS IN THE SOFTWARE.
```

## seuif97 2.3.8, MIT

```text
The MIT License (MIT)

Copyright © 2023 Cheng Maohua

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

## zmij 1.0.23, MIT

```text
Permission is hereby granted, free of charge, to any
person obtaining a copy of this software and associated
documentation files (the "Software"), to deal in the
Software without restriction, including without
limitation the rights to use, copy, modify, merge,
publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software
is furnished to do so, subject to the following
conditions:

The above copyright notice and this permission notice
shall be included in all copies or substantial portions
of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
DEALINGS IN THE SOFTWARE.
```
