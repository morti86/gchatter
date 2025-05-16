## Unmaintained: use aivchat instead

# gchatter
AI voice chat appliation - talk with a selected AI chat using your microphone and headphones.
- either type your question or record it

## Setup
### Requirements
- GTK 4
- Vulkan support for Whisper
- Whisper model onnx file
- Api keys for online models

## TODO
This is still WIP, so there are a few things needed to complete.
- Integrating OCR (Leptess, PaddleOCR) so it is possible to quickly put questions grabbed from the screen or attach a file to a question
- Proper handling of AI chat context for longer conversations.
- More testing and improvements on the overall stability, occasional deadlocks and other issues involving concurrency and it pitfalls.
