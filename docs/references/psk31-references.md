# PSK-31 Reference Materials

Resources collected while investigating the P7 encoder phase flip timing bug.

## Primary Specification

**G3PLX Original Paper** (Peter Martinez, G3PLX)
- Title: "PSK31: A new radio-teletype mode with a traditional philosophy"
- PDF: http://det.bi.ehu.es/jtpjatae/pdf/p31g3plx.pdf
- Originally published in RadCom (RSGB), December 1998 / January 1999
- Note: PDF is binary-encoded and may not render in all viewers

**ARRL PSK-31 Spec**
- https://www.arrl.org/psk31-spec
- Covers Varicode encoding, QPSK modes, and convolutional coding
- Mentions cosine filtering: "In order to eliminate splatter from the phase-reversals
  inherent to PSK, the output is cosine-filtered before reaching the transmitter audio input."
- Does NOT go into detail on the timing of the phase flip relative to symbol boundary

## Technical Descriptions

**fldigi PSK Mode Description** (W1HKJ)
- https://www.w1hkj.org/modes/psk.htm
- Key quote: "all these modes also include 100% raised-cosine amplitude modulation (ASK)
  at the symbol rate, which reduces the power to zero at the phase change."
- Confirms the envelope must reach zero *at* the phase change, not after it

**Quick Look at a BPSK31 Signal** (Baltic Lab)
- https://baltic-lab.com/2012/11/quick-look-at-a-bpsk31-signal/
- Oscilloscope captures showing the 180° phase shift and smooth amplitude transition
- Confirms "the phase shifting occurs at an amplitude minimum"
- Good visual reference for what a correct waveform looks like

**About PSK31** (bpsk31.com)
- https://bpsk31.com/about/
- General overview and Varicode table; not technical on shaping

## DSP Background

**Pulse Shaping** (PySDR)
- https://pysdr.org/content/pulse_shaping.html
- General raised cosine pulse shaping theory

**Raised Cosine Pulse Shaping** (GaussianWaves)
- https://www.gaussianwaves.com/2018/10/raised-cosine-pulse-shaping/
- Mathematical treatment of raised cosine filters

**BPSK Transmitter Theory** (Lloyd Rochester)
- https://lloydrochester.com/post/dsp/psk-transmit-theory/

**WPI Lab: BPSK Modulator**
- https://schaumont.dyn.wpi.edu/ece4703b21/lab5.html
- Practical BPSK implementation notes

## Key Finding re: P7

The W1HKJ and Baltic Lab sources both confirm the correct behavior:
the amplitude must be **zero at the moment of phase change**, not before or after.
The current encoder flips the phase at the start of the symbol where the envelope is 1.0.
The fix requires the phase flip to coincide with the envelope zero crossing (symbol midpoint
or symbol boundary depending on implementation approach).
