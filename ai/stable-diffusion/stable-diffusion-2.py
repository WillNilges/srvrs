# This example was uncerimoniously lifted from: https://huggingface.co/stabilityai/stable-diffusion-2

from diffusers import StableDiffusionPipeline, EulerDiscreteScheduler
import argparse
import torch
from os.path import exists

parser = argparse.ArgumentParser()
parser.add_argument('--prompt', dest='prompt', required=True, type=str, help='A prompt, or a path to a prompt')
parser.add_argument('--device', dest='device', required=True, type=str, help='The device to run the model on')
parser.add_argument('--output', dest='output', required=False, type=str, help='Path to save the final image to')
args = parser.parse_args()

model_id = "stabilityai/stable-diffusion-2"

# Use the Euler scheduler here instead
print('Loading scheduler')
scheduler = EulerDiscreteScheduler.from_pretrained(model_id, subfolder="scheduler")
print('Loading SD Pipeline')
pipe = StableDiffusionPipeline.from_pretrained(model_id, scheduler=scheduler, torch_dtype=torch.float16)
print(f'Running with {args.device}')
pipe = pipe.to(f'{args.device}')

output = 'output.png'
if args.output is not None:
    output = args.output

prompt = ''
if exists(args.prompt):
    with open(args.prompt) as prompt_file:
        prompt = prompt_file.read()
else:
    prompt = args.prompt

image = pipe(prompt).images[0]
image.save(output)
