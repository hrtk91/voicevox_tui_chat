# VoiceVox Chat

## 説明

標準入出力で AI と会話することができます。  
AI からの応答は VoiceVox による合成音声で応答させることができます。

## 必要なもの

- OpenAI API Key

OpenAI の API を利用してチャットに応答させています。

- 起動中の VoiceVox Engine

この辺が楽
[docker](https://hub.docker.com/r/voicevox/voicevox_engine)

## 使い方

`.env`に以下の Key を設定してください
|Key | 説明 |
|---|---|
|OPENAI_API_KEY|OpenAI の API Key|
|VOICEVOX_ENGINE_URL|VoiceVox Engine を稼働させている URL。http[s]://{ip}:{port}形式にしてね|

起動すれば OK
