[![Build Status](https://travis-ci.org/xliiv/dashboard.svg?branch=master)](https://travis-ci.org/xliiv/dashboard)

# NOTE:

This project is inspired by Tipboard (http://allegro.tech/tipboard/)


**FOR NOW THIS IS A TOY PROJECT, I DID THIS FOR LEARNING RUST**


# Live demo

To see Dashboard in action go here:

* [Demo](http://85.255.1.138:8000/)

To see other possible dashboards (**NOT YET AVAILABLE IN ABOVE DEMO**)


* [Dashboard with single tile (tile-value)](http://85.255.1.138/components/dashboard-toolkit/demo/dashboards/single-tile-value.html)
* [Dashboard with single tile (tile-chart)](http://85.255.1.138/components/dashboard-toolkit/demo/dashboards/single-tile-chart.html)
* [Dashboard with 2x4 layout](http://85.255.1.138/components/dashboard-toolkit/demo/dashboards/2x4.html)
* [Dashboard with advanced splitting feature](http://85.255.1.138/components/dashboard-toolkit/demo/dashboards/split-demo.html)


# Running own Dashboard

See [xliiv/dashboard](https://hub.docker.com/r/xliiv/dashboard/) Docker Hub page for details.




# Hack / Develop / Contribute

* install rust (https://www.rust-lang.org/en-US/install.html)
* bower (https://bower.io/#install-bower)
* redis-server


Be aware that tiles components lay in diffrent repository, which is [\<dashboard-toolkit\>](https://github.com/xliiv/dashboard-toolkit)


### Ubuntu:

```
git clone https://github.com/xliiv/dashboard.git
docker run -d redis
cd dashboard/src/static
bower install
cd ../..
# optionally edit dashboard.env file (to set redis server, for example)
cargo run
```

Now, visit http://localhost:8000/ in browser
