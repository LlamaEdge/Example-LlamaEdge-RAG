# Run the RAG applications with LlamaEdge

## Setup

- Install `WasmEdge Runtime`

  ```console
  curl -sSf https://raw.githubusercontent.com/WasmEdge/WasmEdge/master/utils/install.sh | bash -s -- -v 0.13.5  --plugins wasi_nn-ggml wasmedge_rustls
  ```

- Start Qdrant docker container

  ```console
  # Pull the Qdrant docker image
  docker pull qdrant/qdrant

  # Create a directory to store Qdrant data
  mkdir qdrant_storage

  # Run Qdrant service
  docker run -p 6333:6333 -p 6334:6334 -v $(pwd)/qdrant_storage:/qdrant/storage:z qdrant/qdrant
  ```

- Start `llama-api-server`

  **Note that use `improve-rag-endpoints` branch of LlamaEdge to compile `llama-api-server.wasm`.**

  ```bash
  wasmedge --dir .:. --nn-preload default:GGML:AUTO:Llama-2-7b-chat-hf-Q5_K_M.gguf llama-api-server.wasm \
    --prompt-template llama-2-chat \
    --ctx-size 4096 \
    --qdrant-url http://127.0.0.1:6333 \
    --qdrant-collection-name "paris" \
    --qdrant-limit 3
  ```

## Chat using `curl` command

- Upload the chunks of `paris.txt` via `/v1/embeddings` endpoint. Download the paris.json [here](https://github.com/LlamaEdge/Example-LlamaEdge-RAG/blob/main/paris.json).

    ```bash
   curl -s -X POST http://localhost:8080/v1/embeddings \
    -H 'accept:application/json' \
    -H 'Content-Type: application/json' \
    -d @paris.json
    ```

    If the command runs successfully, the embeddings for the document chunks will be persisted in the Qdrant collection `paris` as well as returned in the response.

- Interact with the RAG app via the Chatbot UI

Then, you can open http://localhost:8080 in the browser to chat with the RAG.

![image](https://github.com/LlamaEdge/Example-LlamaEdge-RAG/assets/45785633/e8a2d929-b1b1-4689-9bce-972d8c88f8aa)


- Query a user input via the`/v1/chat/completions` endpoint

    ```bash
    curl -s -X POST http://localhost:8080/v1/chat/completions \
        -H 'accept:application/json' \
        -H 'Content-Type: application/json' \
        -d '{"messages":[{"role":"user","content":"What is the location of Paris, France on the Seine River?\n"}],"model":"llama-2-7b","stream":false}'
    ```

    If the command runs successfully, you will see the following output:

    ```console
    {"id":"e6219b85-0453-407b-8737-f525fe15aa27","object":"chat.completion","created":1709286513,"model":"dummy-chat-model","choices":[{"index":0,"message":{"role":"assistant","content":"According to the provided text, Paris is situated along the Seine River some 233 miles (375 km) upstream from the river’s mouth on the English Channel (La Manche). Therefore, the location of Paris, France on the Seine River is approximately 233 miles (375 km) upstream from the river's mouth."},"finish_reason":"stop"}],"usage":{"prompt_tokens":389,"completion_tokens":78,"total_tokens":467}}
    ```
