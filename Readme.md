# Mio Ring

Memorized Input-Output as Ring, or `mio-ring` for short, is a information integration and processing project designated to collect, sieve and comb through everyday multimedia data flow of an average laptop-based modern human. Oriented with screenshots, voice memo and recording, `mio-ring` provides handy utilities to process the data such as OCR, simple annotations, speech-to-text, and so on, to serialize and summarize your digital life. All of the above happens locally, without any data upload to any server that is not governed beneath your choice.

## Roadmap

Sounds like a dream too ambitious. Maybe let's just start with a simple screenshot software.

## Current behavior

Clone the repo and run `cargo run --release`. At the moment the program starts, a screenshot should done, then a window should be prompted to ask for instructions on how to crop. After you've done, press return to confirm, or shift+return to save fullscreen. Finally, you'll find the screenshot at `~/Library/Caches/LitiaEeloo.MioRing/...`.