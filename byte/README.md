# Byte

Byte is our stream companion, and hopefully chat bot!
They are a work in progress, but we hope to have them up and running soon!

## Plan for Audio Input

- Record audio in chunks
- Run chunks through Whisper to transcribe
- Detect "wake word" (likely Hey Byte) in transcribed text
- If wake word is detected, send transcript to LLM for processing
- Run LLM response through Text to Speech and play it on stream (both for Corey and guests)

## Later Plans

- Add commands that Byte can do
- Add Twitch Chat bot from stream interaction
- Add Discord bot for Discord interaction
- Add Database to act as 'brain'/memory for Byte
  - Idea being he can record things that happen on stream and then recall them later
