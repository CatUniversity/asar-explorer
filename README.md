# asar-explorer

Trait to unpack an electron `.asar` file which implements [`std::io::Read`].


### Usage

Given an asar file named `foo.asar`

```rust
use std::fs::File;

use asar_explorer::Asar;

fn main() -> std::io::Result<()> {
    let mut file = File::open("foo.asar");
    let headers = file.get_headers()?;
    file.unpack_files(&headers, "./foo", None)?;

    Ok(())
}
```

The above sample will unpack all files into a relative directory called `foo`.

And that's about it.