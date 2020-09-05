import * as React from 'react';

import SimpleTTS = require("simpletts");
 
interface Voice {
    name: string;
    gender: "female" | "male";
}
 
interface Options {
    text: string;
    volume?: number;
    speed?: number;
    voice?: Voice | string;
}
 
const tts = new SimpleTTS();
 
tts.getVoices().then((voices: Array<Voice>) => {
 
    return tts.read({
        "text": "test",
        "voice": voices[0]
    });
 
}).then((options: Options) => {
    console.log(options);
}).catch((err: Error) => {
    console.log(err);
});

export default Audio; 