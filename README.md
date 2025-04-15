# gchatter
AI voice chat appliation - talk with a selected AI chat using your microphone and headphones.
- either type your question or record it
- can also record from the output, so can be used to cheat interviews ^_^


## Setup
### Requirements
- GTK 4
- Vulkan support for Whisper
- Whisper model onnx file
- Api keys for online models

## TODO
This is still WIP, so there are a few things needed to complete.
- Integraging Elevenlabs client library for reading the output of the AI. The main consideration is the buffering of data because it should not be sent word by word, but sentence by sentence and the output of a chat needs to be properly cleaned up. It adds even more complexity as the reader must be run in a separate thread and the communicatiom has to be done through MPSC channel.
- Integrating OCR (Leptess, PaddleOCR) so it is possible to quickly put questions grabbed from the screen or attach a file to a question
- Proper handling of AI chat context for longer conversations.
- More testing and improvements on the overall stability, occasional deadlocks and other issues involving concurrency and it pitfalls.
