[![Build Status](https://travis-ci.org/xliiv/dashboard.svg?branch=master)](https://travis-ci.org/xliiv/dashboard)

# NOTE:

This project is inspired by Tipboard (http://allegro.tech/tipboard/)


**FOR NOW THIS IS A TOY PROJECT, I DID THIS FOR LEARNING RUST**


## Screens

<img src="https://raw.githubusercontent.com/xliiv/dashboard/master/_demo/screens/listing.png" width="100%" />
<img src="https://raw.githubusercontent.com/xliiv/dashboard/master/_demo/screens/demo-image3.png" width="100%" />

<details>
 <summary>More screens ..</summary>
<img src="https://raw.githubusercontent.com/xliiv/dashboard/master/_demo/screens/demo-image1.png" width="100%" />
<img src="https://raw.githubusercontent.com/xliiv/dashboard/master/_demo/screens/demo-image2.png" width="100%" />
<img src="https://raw.githubusercontent.com/xliiv/dashboard/master/_demo/screens/demo-image4.png" width="100%" />
<img src="https://raw.githubusercontent.com/xliiv/dashboard/master/_demo/screens/demo-image5.png" width="100%" />
</details>



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
