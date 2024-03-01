#!/bin/bash

# * check if docker is installed
if command -v docker &> /dev/null
then
    printf "[+] Detecting Docker ...\n"
    result=$(docker --version)
    printf "    %s\n" "$result"
else
    printf "Docker is required for running this example.\n"
    exit 1
fi
printf "\n"


# # * install WasmEdge with wasi-nn_ggml plugin
printf "[+] Install WasmEdge with wasi-nn_ggml plugin ...\n\n"
if curl -sSf https://raw.githubusercontent.com/WasmEdge/WasmEdge/master/utils/install.sh | bash -s -- -v 0.13.5 --plugins wasi_nn-ggml wasmedge_rustls; then
    source $HOME/.wasmedge/env
    wasmedge_path=$(which wasmedge)
    printf "\n    The WasmEdge Runtime is installed in %s.\n\n" "$wasmedge_path"
else
    printf "    Failed to install WasmEdge\n"
    exit 1
fi
printf "\n"

# * download Llama-2-7b-chat model
file="Llama-2-7b-chat-hf-Q5_K_M.gguf"
url="https://huggingface.co/second-state/Llama-2-7B-Chat-GGUF/resolve/main/Llama-2-7b-chat-hf-Q5_K_M.gguf"

if [ ! -f "$file" ]; then
    printf "[+] Downloading '$file' ...\n\n"
    curl -LO https://huggingface.co/second-state/Llama-2-7B-Chat-GGUF/resolve/main/Llama-2-7b-chat-hf-Q5_K_M.gguf
else
    printf "[+] Using the cached '$file' ...\n\n"
fi

# * download llama-api-server.wasm
printf "[+] Downloading the latest 'llama-api-server.wasm' ...\n\n"
curl -LO https://github.com/LlamaEdge/LlamaEdge/releases/latest/download/llama-api-server.wasm
printf "\n\n"

# * start api server
printf "[+] Starting the LlamaEdge server ...\n\n"
nohup wasmedge --dir .:. --nn-preload default:GGML:AUTO:Llama-2-7b-chat-hf-Q5_K_M.gguf llama-api-server.wasm --prompt-template llama-2-chat --ctx-size 4096 >/dev/null 2>&1 &

wasmedge_pid=$!
printf "    LlamaEdge Server PID: %s\n\n" "$wasmedge_pid"

# wait for the LlamaEdge server to start. Loading the model takes a few seconds.
sleep 5

# * start Qdrant docker container
printf "[+] Starting Qdrant docker container...\n\n"
found=$(docker image list | grep "qdrant/qdrant")
if [ -z "$found" ]; then
    printf "    Pulling the 'qdrant/qdrant' docker image...\n"
    docker pull qdrant/qdrant
fi

if [ ! -d "qdrant_storage" ]; then
    echo "    Creating the 'qdrant_storage' directory for the local storage..."
    mkdir -p qdrant_storage
fi

nohup docker run -p 6333:6333 -p 6334:6334 -v $(pwd)/qdrant_storage:/qdrant/storage:z qdrant/qdrant >/dev/null 2>&1 &

qdrant_pid=$!
printf "    Qdrant Server PID: %s\n\n" "$qdrant_pid"

# wait for the Qdrant docker container to start.
sleep 2

# * use curl to upload a document and generate embeddings
printf "[+] Uploading a document and generating embeddings...\n\n"

response=$(curl -s -X POST http://localhost:8080/v1/rag/document -H 'accept:application/json' -H 'Content-Type: application/json' -d '{"embeddings":{"model":"dummy-embedding-model","input":["Paris, city and capital of France, situated in the north-central part of the country. People were living on the site of the present-day city, located along the Seine River some 233 miles (375 km) upstream from the river’s mouth on the English Channel (La Manche), by about 7600 BCE. The modern city has spread from the island (the Île de la Cité) and far beyond both banks of the Seine.","Paris occupies a central position in the rich agricultural region known as the Paris Basin, and it constitutes one of eight départements of the Île-de-France administrative region. It is by far the country’s most important centre of commerce and culture. Area city, 41 square miles (105 square km); metropolitan area, 890 square miles (2,300 square km).","Pop. (2020 est.) city, 2,145,906; (2020 est.) urban agglomeration, 10,858,874.","For centuries Paris has been one of the world’s most important and attractive cities. It is appreciated for the opportunities it offers for business and commerce, for study, for culture, and for entertainment; its gastronomy, haute couture, painting, literature, and intellectual community especially enjoy an enviable reputation. Its sobriquet “the City of Light” (“la Ville Lumière”), earned during the Enlightenment, remains appropriate, for Paris has retained its importance as a centre for education and intellectual pursuits.","Paris’s site at a crossroads of both water and land routes significant not only to France but also to Europe has had a continuing influence on its growth. Under Roman administration, in the 1st century BCE, the original site on the Île de la Cité was designated the capital of the Parisii tribe and territory. The Frankish king Clovis I had taken Paris from the Gauls by 494 CE and later made his capital there.","Under Hugh Capet (ruled 987–996) and the Capetian dynasty the preeminence of Paris was firmly established, and Paris became the political and cultural hub as modern France took shape. France has long been a highly centralized country, and Paris has come to be identified with a powerful central state, drawing to itself much of the talent and vitality of the provinces."]},"url":"http://127.0.0.1:6333","collection_name":"paris"}')

printf "    %s" "$response"
printf "\n\n"

# * use curl to query
printf "[+] Querying user's question...\n\n"
printf "    user query: What is the location of Paris, France on the Seine River?\n\n"

response=$(curl -s -X POST http://localhost:8080/v1/rag/query -H 'accept:application/json' -H 'Content-Type: application/json' -d '{"chat_model":"dummy-chat-model","messages":[{"role":"user","content":"What is the location of Paris, France on the Seine River?\n"}],"embedding_model":"dummy-embedding-model","qdrant_url":"http://127.0.0.1:6333","qdrant_collection_name":"paris","limit":3,"stream":false}')

printf "    [Response] %s" "$response"
printf "\n\n"

# * stop the Qdrant docker container
printf "[+] Stopping Qdrant Docker container ...\n\n"
kill $qdrant_pid

# * stop the LlamaEdge server
printf "[+] Stopping LlamaEdge ...\n\n"
kill $wasmedge_pid

printf "[+] Done.\n\n"
exit 0