# DISCLAIMER: Technical demo code — legal wrappers required in production
# SPDX-License-Identifier: MIT
#
# Next.js dashboard for AssetMint demo

FROM node:18-alpine
WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci --silent
COPY . .
EXPOSE 3000
CMD ["npx", "next", "dev", "--port", "3000"]
