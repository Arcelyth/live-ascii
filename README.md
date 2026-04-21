# Live-ASCII

A Live2D Cubism model renderer for terminal. It also support face tracking.

![showcase](./showcase.gif)

## Usage 

You must have the Live2D Cubism SDK Core library: [Live2D Cubism SDK](https://www.live2d.com/en/sdk/about/). <br>
Create a .env file in the project root to specify the path to the SDK directory:
```.env
# Example .env configuration
CubismSDKDir=/path/to/your/CubismSDK/Core/lib
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
## Basic Operations
The arrow keys are used for moving the camera, and the `+` `-` keys are used for zooming in and out.

## Interactions

In order to interact with app. You need to write a `model_name.live.json` file and place it in the same folder as your `model_name.model3.json`. <br>
examples: 
```json
{
  "Version": 1,
  "Name": "your_model_name",
  "Hotkeys": [
    {
      "Action": "Open/Close Motion Panel",
      "Triggers": {
        "Trigger1": "M",
        "Trigger2": "",
        "Trigger3": ""
      }
    },
    {
      "Action": "Open/Close Debug Panel",
      "Triggers": {
        "Trigger1": "D",
        "Trigger2": "",
        "Trigger3": ""
      }
    },
    {
      "Action": "Enable/Disable Physics",
      "Triggers": {
        "Trigger1": "P",
        "Trigger2": "",
        "Trigger3": ""
      }
    },
    {
      "Action": "Open/Close Camera",
      "Triggers": {
        "Trigger1": "C",
        "Trigger2": "",
        "Trigger3": ""
      }
    }
  ]
}
```

## Features in future
- Separate live2d framework to a crate.
- Support processes interaction.
- Enable multiply expressions.
