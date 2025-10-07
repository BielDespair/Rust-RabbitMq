# Processador de Documentos Fiscais em Rust

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![RabbitMQ](https://img.shields.io/badge/Rabbitmq-FF6600.svg?style=for-the-badge&logo=rabbitmq&logoColor=white)

Este microserviço foi projetado para ter elevada performance, atuando como um parser dos XML das notas fiscais de clientes (NF-e, NFC-e, CT-e, Eventos), consumindo de uma fila do `RabbitMQ` e publicando um JSON customizado em outra fila.

## Visão Geral

Este projeto atua como um *worker*. Ele consome mensagens de uma fila do RabbitMQ, utiliza as informações da mensagem para baixar o XML correspondente que está armazenado no MinIO, realiza um parse do documento e, por fim, publica o resultado estruturado em um formato JSON customizado em outra fila do RabbitMQ para consumo por outros sistemas.

O serviço é construído com foco em resiliência e performance, utilizando o runtime assíncrono `tokio` e o parser de baixo nível `quick-xml`.

## Funcionamento

O fluxo de processamento de um documento ocorre nas seguintes etapas:

1.  **Consumo da Fila**: O `consumer.rs` escuta a fila de entrada do RabbitMQ.
2.  **Decodificação da Mensagem**: Uma mensagem é recebida e seu conteúdo JSON é decodificado para obter o nome do arquivo XML e metadados (`company_id`, `org_id`).
3.  **Download do Objeto**: O `minio_client.rs` é acionado para baixar o arquivo XML do bucket Minio.
4.  **Identificação e Parse**: O `nfe_parser.rs` analisa o XML para identificar o tipo de documento (NF-e, Lote, Evento, etc.).
5.  **Mapeamento para Structs**: Com base no tipo, o parser percorre o XML e mapeia os dados para as `structs` definidas em `nfes.rs`.
6.  **Serialização para JSON**: A `struct` final, contendo todos os dados extraídos, é serializada para uma string JSON.
7.  **Publicação do Resultado**: O JSON é publicado na fila de saída do RabbitMQ.
8.  **Confirmação (ACK/NACK)**: Se todas as etapas forem concluídas com sucesso, a mensagem original é confirmada (`ack`). Em caso de qualquer falha, a mensagem é rejeitada (`reject`), sendo enviada para uma Dead Letter Queue.

## Como Executar
### Pré-requisitos
-   Rust 1.89.0 >
-   Cargo 1.89.0 >
-   Acesso a uma instância do RabbitMQ.
-   Acesso a uma instância do Minio (ou outro S3 compatível).

### Configuração

Crie um arquivo `.env` e preencha as variáveis de ambiente listadas na sessão [**Variáveis de Ambiente**](#variáveis-de-ambiente).

A variável `RABBITMQ_NUM_CHANNELS` define quantos canais de consumo simultâneos serão abertos com o RabbitMQ, permitindo processar múltiplas mensagens em paralelo. (Pense em threads)

> **Observação:**  
> Caso deseje publicar mensagens utilizando a *default exchange* do RabbitMQ, defina `RABBITMQ_EXCHANGE` como vazio (`""`).  
> Nesse modo, **a chave de roteamento (`RABBITMQ_ROUTING_KEY`) deve ser igual ao nome da fila (`RABBITMQ_PUBLISH_QUEUE`)**, pois o RabbitMQ roteará a mensagem diretamente para a fila de mesmo nome.  
>  
> Exemplo:
> ```env
> RABBITMQ_EXCHANGE=
> RABBITMQ_PUBLISH_QUEUE=xml_queue
> RABBITMQ_ROUTING_KEY=xml_queue
> ```




## Executando Localmente

```bash
# Clone o repositório
git clone git@bitbucket.org:fgfconsult/simplex-consumidor-xml.git
cd simplex-consumidor-xml

# Compile e execute em modo de desenvolvimento
cargo run
# Compile e execute em modo de produção (otimizado)
cargo run --release
```

## Executando com Docker

O projeto inclui um `Dockerfile` com as configurações necessárias para executar um container.

```bash
docker build -t simplex-consumidor-xml .
docker run --rm --env-file .env simplex-consumidor-xml

```

## Variáveis de Ambiente

As seguintes variáveis de ambiente são necessárias para a execução do serviço.

| Variável | Descrição | Exemplo |
| :--- | :--- | :--- |
| **Logging** | | |
| `RUST_LOG` | Nível de log da aplicação (trace, debug, info, warn, error). | `info` |
| **Minio (S3 Storage)** | | |
| `MINIO_ENDPOINT` | Endpoint do servidor Minio/S3. | `localhost:9000` |
| `MINIO_ACCESS_KEY` | Chave de acesso do Minio/S3. | `minioadmin` |
| `MINIO_SECRET_KEY` | Chave secreta do Minio/S3. | `minioadmin` |
| `MINIO_BUCKET_NAME`| Nome do bucket onde os XMLs estão armazenados. | `nfe-xmls` |
| **RabbitMQ (Message Queue)** | | |
| `RABBITMQ_HOST` | Host do servidor RabbitMQ. | `localhost` |
| `RABBITMQ_PORT` | Porta do servidor RabbitMQ. | `5672` |
| `RABBITMQ_USER` | Usuário de acesso ao RabbitMQ. | `guest` |
| `RABBITMQ_PASSWORD`| Senha do usuário. | `guest` |
| `RABBITMQ_VHOST` | Virtual host a ser utilizado. | `/` |
| `RABBITMQ_EXCHANGE`| Nome do exchange a ser utilizado. | `nfe_exchange` |
| `RABBITMQ_CONSUME_QUEUE`| Nome da fila de onde as mensagens serão consumidas. | `xml_para_processar` |
| `RABBITMQ_PUBLISH_QUEUE`| Nome da fila onde os resultados JSON serão publicados. | `json_processado` |
| `RABBITMQ_ROUTING_KEY`| Chave de roteamento para publicação e binding das filas. | `nfe.json` |
| `RABBITMQ_NUM_CHANNELS`| Número de canais de consumo a serem abertos. | `10` |


## Autores

-   **Gabriel Alves Leonel** - *Desenvolvimento Inicial*

## Responsável Técnico

Este projeto é atualmente mantido por:

**Gabriel Alves Leonel**  
gabrielleonel@ctributaria.com.br
