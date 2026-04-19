# Live-ASCII

A Live2D Cubism model renderer for terminal. It also support face tracking.

![showcase](./showcase.gif)

## Usage 

You must have the Live2D Cubism SDK Core library: [Live2D Cubism SDK](https://www.live2d.com/en/sdk/about/). <br>
Create a .env file in the project root to specify the path to the SDK directory:
```.env
# Example .env configuration
CubismSDKDir=/path/to/your/CubismSDK/Core/lib/macos
```

```bash
cargo run --release -- ./path/to/model.model3.json
```

You can download and try Live2D sample model [here](https://www.live2d.com/en/learn/sample/).

```bash
# Run with camera tracking enabled
cargo run --release -- ./path/to/model.model3.json --camera
```

Note: *For face tracking, ensure [OpenSeeFace](https://github.com/emilianavt/OpenSeeFace) is running and sending data to the default UDP port (11573).*

## Operations and Debug
Press `m` to choose the motion which you want to play. <br>
Press `p` to show the debug panel. Press `1`-`6` to display different parameters in debug panel.

## Features in future
- Separate live2d framework to a crate
- Support processes interaction
- Complete handle `live.json` file for customizing actions
