FROM docker.io/nvidia/cuda:11.4.1-cudnn8-devel-ubuntu20.04

RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get -y install git python3-pip python3

WORKDIR /root

RUN pip install diffusers transformers accelerate scipy safetensors
RUN pip install -U xformers
COPY cache-models.py .
RUN python3 cache-models.py && echo "Model Downloaded."

COPY . .

WORKDIR /workdir

ENTRYPOINT python3 stable-diffusion-2.py --prompt $SD2_PROMPT --device $SD2_DEVICE
