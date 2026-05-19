# Custom Wake Word Training Guide

EyeForge uses Picovoice Porcupine for offline wake word detection. You need a **Picovoice AccessKey** first — it's required even for built-in keywords.

## Getting an AccessKey

1. Go to [Picovoice Console](https://console.picovoice.ai/)
2. Click **Sign Up** (Google / GitHub login supported)
3. After logging in, you'll find your **AccessKey** on the console home page (starts with your email, e.g., `your@email.com/...`)
4. Copy the key and paste it into EyeForge Settings → General → **Picovoice AccessKey**

> The free AccessKey is valid forever — no payment required.

## Built-in Keywords

The following keywords work out of the box:

`computer`, `hey google`, `alexa`, `hey siri`, `porcupine`, `grasshopper`, `terminator`, `bumblebee`, `picovoice`, `blueberry`

Just type them in Settings → General → Wake Words.

## Training a Custom Wake Word

### Step 1: Create a Picovoice Account

Same as above — log into the [Picovoice Console](https://console.picovoice.ai/).

### Step 2: Create a Custom Wake Word

1. In the left sidebar, select **Wake Word** → **Create**
2. Enter your desired wake word (e.g., "Hey Forge", "Computer")
3. Select the language
4. Click **Create** — the system will generate an audio model for your wake word

> Free accounts can create **1 custom wake word per month** (resets monthly).

### Step 3: Download the .ppn File

1. After creation, click **Download** in the wake word list
2. Select **Porcupine** as the platform
3. Select **Python** as the version
4. Download the generated `.ppn` file

### Step 4: Add to Project

1. Place the `.ppn` file in the `EyeForge/` directory (or anywhere)
2. In Settings → General → Wake Words, enter the **absolute path** to the `.ppn` file

Example configuration:

```
C:\Users\YourName\EyeForge\hey-forge.ppn
```

> Custom wake words require a `.ppn` file path. Built-in keywords can be entered by name. Separate multiple keywords with commas.

### Step 5: Test

1. Make sure the Picovoice AccessKey field is filled in
2. Enable wake word detection
3. Speak your trained wake word
4. The floating input window should appear and start recording automatically

## Notes

- **AccessKey is required** — even for built-in keywords like "computer"
- Free Picovoice accounts can create **1 custom wake word per month** (built-in keywords are unlimited)
- `.ppn` files are platform-specific — always download the Python version
- Once trained, wake words work offline permanently
- For more wake words, consider upgrading to a Picovoice paid plan
