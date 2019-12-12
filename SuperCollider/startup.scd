s.boot;

s.waitForBoot({
var currentFreq, currentVolume, currentType, typeText, cpuText, font,
currentGenerator, setFreq, setVol, setType, cpuInfo, leftText, rightText;

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


// GUI
w = Window.new("TONE GENERATOR", Window.availableBounds);
w.view.decorator = d = FlowLayout.new( w.view.bounds, 10@10, 20@5 );
font = Font([
        "Helvetica-Bold",
        "Helvetica",
        "Monaco",
        "Arial",
        "Gadget"
    ].choose, 20);

c = Stethoscope.new(s, 1, view:w.view);

f = FreqScopeView.new(w, 350@250);
f.active_(true);

d.nextLine;

leftText = FlowView(w, 250@150);
rightText = FlowView(w, Rect(0, 0, 550, 500));


currentType = "SineOscillator";
typeText = StaticText(leftText, 350@32);
typeText.string = "Type:" + currentType;
typeText.font = font;

d.nextLine;

currentFreq = 220;
a = StaticText(leftText, 250@32);
a.string = "Frequency:" + currentFreq ++ "hz";
a.font = font;

d.nextLine;

currentVolume = 20;
b = StaticText(leftText, 200@32);
b.string = "Volume:" + currentVolume ++ "%";
b.font = font;

cpuText = StaticText(rightText, Rect(0, 0, 550, 500));
cpuText.font = font;
cpuText.string = "Booting";
cpuText.align = \topLeft;

// Generators
currentGenerator = Synth.new(\SineOscillator);

// OSC Response
n = NetAddr.new("192.168.1.219", 57120);
setFreq = OSCFunc({ arg msg, time, addr, recvPort;
	// [msg, time, addr, recvPort].postln;
	currentFreq = msg.at(1).asInteger;
	currentGenerator.set(\freq, currentFreq);
	{ a.string = "Frequency:" + currentFreq ++ "hz" }.defer;
}, '/frequency', n);

setFreq = OSCFunc({ arg msg, time, addr, recvPort;
	currentVolume = msg.at(1);
	currentGenerator.set(\mul, currentVolume);
	{ b.string = "Volume:" + (currentVolume * 100).asInteger ++ "%" }.defer;
}, '/volume', n);

setFreq = OSCFunc({ arg msg, time, addr, recvPort;
	var selectedOsc;
	selectedOsc = [
		"SineOscillator",
		"TriangleOscillator",
		"SawOscillator",
		"PulseOscillator",
		"NoiseGenerator" ].at(msg.at(1));
	if (currentType != selectedOsc, {
		"Not the same!".postln;
		currentGenerator.free;
		currentType = selectedOsc;
		currentGenerator = Synth.new(selectedOsc, [\freq, currentFreq, \mul, currentVolume]);
		{ typeText.string = "Type:" + currentType; }.defer;
	});
	selectedOsc.postln;
}, '/type', n);

thisProcess.addOSCRecvFunc(setFreq);

// Computer Status
r = Routine({
	fork {
		loop {
			p = Pipe.new(thisProcess.nowExecutingPath.dirname ++ "/cpuInfo.sh", "r");
			l = p.getLine;
			cpuInfo = "";
			while({l.notNil}, {cpuInfo = cpuInfo + "\n" ++ l; l = p.getLine;});
			p.close;
			{cpuText.string = cpuInfo}.defer;
			1.0.yield;
		}
	}
});
SystemClock.play(r);

// Shutdown
w.onClose = {
	c.free;
	f.kill;
	setFreq.free;
	currentGenerator.free;
};

w.front;
})
