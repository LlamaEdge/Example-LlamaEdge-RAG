# Interact with LlamaEdge RAG endpoints using `curl`

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
  wasmedge --dir .:. --nn-preload default:GGML:AUTO:Llama-2-7b-chat-hf-Q5_K_M.gguf llama-api-server.wasm --prompt-template llama-2-chat --ctx-size 4096 --qdrant-url http://127.0.0.1:6333 --qdrant-collection-name "paris" --qdrant-limit 3
  ```

## Chat using `curl` command

- Upload the chunks of `paris.txt` via `/v1/rag/document` endpoint

    ```bash
    curl -s -X POST http://localhost:8080/v1/embeddings \
        -H 'accept:application/json' \
        -H 'Content-Type: application/json' \
        -d '{"model":"dummy-embedding-model","input":["Paris, city and capital of France, situated in the north-central part of the country. People were living on the site of the present-day city, located along the Seine River some 233 miles (375 km) upstream from the river’s mouth on the English Channel (La Manche), by about 7600 BCE. The modern city has spread from the island (the Île de la Cité) and far beyond both banks of the Seine.","Paris occupies a central position in the rich agricultural region known as the Paris Basin, and it constitutes one of eight départements of the Île-de-France administrative region. It is by far the country’s most important centre of commerce and culture. Area city, 41 square miles (105 square km); metropolitan area, 890 square miles (2,300 square km).","Pop. (2020 est.) city, 2,145,906; (2020 est.) urban agglomeration, 10,858,874.","For centuries Paris has been one of the world’s most important and attractive cities. It is appreciated for the opportunities it offers for business and commerce, for study, for culture, and for entertainment; its gastronomy, haute couture, painting, literature, and intellectual community especially enjoy an enviable reputation. Its sobriquet “the City of Light” (“la Ville Lumière”), earned during the Enlightenment, remains appropriate, for Paris has retained its importance as a centre for education and intellectual pursuits.","Paris’s site at a crossroads of both water and land routes significant not only to France but also to Europe has had a continuing influence on its growth. Under Roman administration, in the 1st century BCE, the original site on the Île de la Cité was designated the capital of the Parisii tribe and territory. The Frankish king Clovis I had taken Paris from the Gauls by 494 CE and later made his capital there.","Under Hugh Capet (ruled 987–996) and the Capetian dynasty the preeminence of Paris was firmly established, and Paris became the political and cultural hub as modern France took shape. France has long been a highly centralized country, and Paris has come to be identified with a powerful central state, drawing to itself much of the talent and vitality of the provinces."]}'
    ```

    If the command runs successfully, you will see the following output:

    ```console
    The embeddings for the document chunks has been computed and persisted in the Qdrant collection 'paris'.
    ```

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
