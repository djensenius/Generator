(
var currentFreq, currentVolume, currentType, typeText, font,
currentGenerator, setFreq, setVol, setType;

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

f = FreqScopeView.new(w, 100@250);
f.active_(true);

d.nextLine;

currentType = "SineOscillator";
typeText = StaticText(w, 350@24);
typeText.string = "Type:" + currentType;
typeText.font = font;

d.nextLine;

currentFreq = 220;
a = StaticText(w, 250@24);
a.string = "Frequency:" + currentFreq ++ "hz";
a.font = font;

d.nextLine;

currentVolume = 20;
b = StaticText(w, 200@24);
b.string = "Volume:" + currentVolume ++ "%";
b.font = font;

// Generators
currentGenerator = Synth.new(\SineOscillator);

// OSC Response
n = NetAddr.new("192.168.1.219", 57120);
setFreq = OSCFunc({ arg msg, time, addr, recvPort;
	[msg, time, addr, recvPort].postln;
	currentFreq = msg.at(1).asInteger;
	currentGenerator.set(\freq, currentFreq);
	{ a.string = "Frequency:" + currentFreq ++ "hz" }.defer;
}, '/frequency', n);

setFreq = OSCFunc({ arg msg, time, addr, recvPort;
	[msg, time, addr, recvPort].postln;
	currentVolume = msg.at(1);
	currentGenerator.set(\mul, currentVolume);
	{ b.string = "Volume:" + (currentVolume * 100).asInteger ++ "%" }.defer;
}, '/volume', n);

setFreq = OSCFunc({ arg msg, time, addr, recvPort;
	var selectedOsc;
	[msg, time, addr, recvPort].postln;
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

// Shutdown
w.onClose = {
	c.free;
	f.kill;
	setFreq.free;
	currentGenerator.free;
};

w.front;
)
