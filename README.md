[![Build Status](https://travis-ci.org/xliiv/dashboard.svg?branch=master)](https://travis-ci.org/xliiv/dashboard)

# NOTE:

This project is inspired by Tipboard (http://allegro.tech/tipboard/)


**FOR NOW THIS IS A TOY PROJECT, I DID THIS FOR LEARNING RUST**


# Live demo

To see dashboards in action go here

* [Dashboard with single tile (tile-value)](http://85.255.1.138/components/dashboard-toolkit/demo/dashboards/single-tile-value.html)
* [Dashboard with single tile (tile-chart)](http://85.255.1.138/components/dashboard-toolkit/demo/dashboards/single-tile-chart.html)
* [Dashboard with 2x8 layout](http://85.255.1.138/components/dashboard-toolkit/demo/dashboards/2x8.html)
* [Dashboard with advanced splitting feature](http://85.255.1.138/components/dashboard-toolkit/demo/dashboards/split-demo.html)


# Running own dashboard

* install [Docker](https://docs.docker.com/engine/installation/)
* install [Docker-compose](https://docs.docker.com/compose/install/)
* make a file `docker-compose.yml` with this content:

```
version: '2'
services:
  web:
    image: xliiv/dashboard
    ports:
    - "8000:8000"
    - "8001:8001"
    links:
    - redis
    environment:
        - DASHBOARD_IP_PORT=0.0.0.0:8000
        - DASHBOARD_WEBSOCKET_IP_PORT=0.0.0.0:8001
        - DASHBOARD_FRONT_WEBSOCKET_IP_PORT=0.0.0.0:8001
        - DASHBOARD_REDIS_IP_HOST=redis:6379
        - DASHBOARD_DASHBOARD_TOKEN=change-me
  redis:
    image: redis
```
* customize environment variables
* run command `docker-compose up`
* send data to example dashboard by running [script](https://raw.githubusercontent.com/xliiv/dashboard/master/src/dashboards/feed.py)

## Customize dashboard

* TODO :)
* example running demo with dashboard customization
* link and explain [\<dashboard-toolkit\>](https://github.com/xliiv/dashboard-toolkit)


# Hack / Develop / Contribute

* install rust (https://www.rust-lang.org/en-US/install.html)
* bower (https://bower.io/#install-bower)
* redis-server


### Ubuntu:

```
git clone https://github.com/xliiv/dashboard.git
docker run -d redis
cd dashboard/src/static
bower install
cd ../..
# optionally edit dashboard.env file (to set redis server, for example)
cargo run
# to load example dashboard data, from another terminal run:
python3 src/dashboards/feed.py  # needs `request` lib
```

Now, visit http://localhost:8000/ in browser to see example dashboard
