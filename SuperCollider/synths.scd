(
SynthDef("SineOscillator", {| freq = 220, mul = 0.5 |
    Out.ar(0, Mix(SinOsc.ar(freq, 0, mul)));
}).store;

SynthDef("SawOscillator", {| freq = 220, mul = 0.5 |
    Out.ar(0, Mix(LFSaw.ar(freq, 0, mul)));
}).store;

SynthDef("TriangleOscillator", {| freq = 220, mul = 0.5 |
    Out.ar(0, Mix(LFTri.ar(freq, 0, mul)));
}).store;

SynthDef("PulseOscillator", {| freq = 220, mul = 0.5 |
	Out.ar(0, Mix(Pulse.ar([freq, freq*2, freq*3],  SinOsc.kr(mul).range(0.01, 0.99), mul)));
}).store;

SynthDef("NoiseGenerator", {| freq = 220, mul = 0.5 |
	Out.ar(0, Mix(LFNoise1.ar(freq, mul * 2, 0)));
}).store;

)