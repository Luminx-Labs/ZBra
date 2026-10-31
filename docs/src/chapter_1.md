# ZBra

<section class="velox-hero" aria-labelledby="hero-title">
    <div class="velox-hero__copy">
        <p class="velox-eyebrow"><span class="velox-signal"></span> research-grade chess systems</p>
        <h1 id="hero-title">ZBra<span class="velox-accent">.</span></h1>
        <p class="velox-lede">A precise, fast-moving chess engine foundation for classical search, neural evaluation, and serious experimentation.</p>
        <div class="velox-hero__actions">
            <a class="velox-button velox-button--primary" href="#chapter-1">Enter the guide <span aria-hidden="true">↗</span></a>
            <a class="velox-button velox-button--quiet" href="#the-development-guide">Explore the stack</a>
        </div>
        <div class="velox-metrics" aria-label="Project characteristics">
            <span><strong>Rust</strong> core</span>
            <span><strong>NNUE</strong> ready</span>
            <span><strong>∞</strong> positions</span>
        </div>
    </div>

<div class="velox-hero__board-wrap" aria-label="Animated chess board">
<div class="velox-board-glow"></div>
<div class="velox-board" data-chess-board role="img" aria-label="A chess position with an animated knight"></div>
<div class="velox-board-caption"><span class="velox-board-caption__dot"></span> engine / online</div>
</div>
</section>

<div id="chapter-1" class="velox-section-rule" aria-hidden="true"></div>

# The development guide

Welcome to the ZBra development guide. This is the working map for the engine’s architecture, tooling, evaluation systems, and reproducible experiments.

<div class="velox-card-grid">
    <a class="velox-card" href="#the-development-guide">
        <span class="velox-card__index">01</span>
        <h2>Build with intent</h2>
        <p>Understand the workspace, the xtask workflow, and the checks that keep every move deterministic.</p>
        <span class="velox-card__arrow" aria-hidden="true">→</span>
    </a>
    <a class="velox-card" href="#the-development-guide">
        <span class="velox-card__index">02</span>
        <h2>Think in positions</h2>
        <p>Follow the path from board representation to search, evaluation, and neural network tooling.</p>
        <span class="velox-card__arrow" aria-hidden="true">→</span>
    </a>
    <a class="velox-card" href="#the-development-guide">
        <span class="velox-card__index">03</span>
        <h2>Ship with proof</h2>
        <p>Use the release and validation flows to turn experiments into dependable engine artifacts.</p>
        <span class="velox-card__arrow" aria-hidden="true">→</span>
    </a>
</div>
