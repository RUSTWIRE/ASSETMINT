# DISCLAIMER: Technical demo code — legal wrappers required in production
# SPDX-License-Identifier: MIT
FROM node:22-alpine
WORKDIR /app
COPY server.js .
RUN mkdir -p /data
ENV PORT=8900
ENV DB_FILE=/data/metadata.json
EXPOSE 8900
CMD ["node", "server.js"]
