# Changelog

## v0.1.0 (2023-09-24)

### Added or Changed

- rs-exchanges-parser
  - added mexc parser
  - added kucoin parser
  - added gate parser
  - created DB structure
  - worker is parsing all exchanges and saves to DB
- setup and tested Continuous Integration, each release to github will
  - build, test, check for vulnerabilities and warnings in rust code
  - build docker images
  - deploy docker containers to the server

## v0.1.0 (2023-09-22)

### Added or Changed

- Created project structure consisting of the following packages
  - rs-utils - will contain all necessary clients to MongoDB, HTTP requests, websocket connections, Redis caching, etc
  - rs-exchanges-parser - will parse all CEX pairs with AZERO token and save it to local MongoDB database
  - rs-subscan-parser - will parse stake/unstake in subscan.io, withdraw/deposit from known CEX wallets and save it local MongoDB database
  - rs-azero-dev-parser - will parse whale transfers in azero.dev and save it local MongoDB database
  - rs-telegram-feed-bot - will post to telegram channel
    - all the transactions that meet the criteria (i.e. larger than $5,000 or anything else community suggests)
    - weekly/monthly summaries
- Added http_client into rs-utils package
- Added mongodb_client into rs-utils package
- Added print_utils into rs-utils package
