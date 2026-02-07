<script lang="ts">
	interface Props {
		summary: {
			working: number;
			permission: number;
			input: number;
			connecting: number;
		};
		total: number;
	}

	let { summary, total }: Props = $props();

	let trackWidth = $state(0);

	// Calculate blocks to maintain square aspect ratio
	// Height = 28px - 2px(border) - 6px(padding) = 20px
	// 2 rows with 3px gap => (20 - 3) / 2 = 8.5px per block
	// Block width needs to be 8.5px to be square
	// Gap is 3px
	// Stride is 11.5px
	let columns = $derived(Math.max(1, Math.floor((trackWidth - 6) / 11.5)));
	let totalBlocks = $derived(columns * 2);

	let blocks = $derived.by(() => {
		if (total === 0) return { working: 0, permission: 0, input: 0, connecting: 0 };

		const counts = [summary.working, summary.permission, summary.input, summary.connecting];
		const percentages = counts.map((c) => (c / total) * totalBlocks);
		const integerParts = percentages.map((p) => Math.floor(p));
		const remainders = percentages.map((p, i) => p - integerParts[i]);

		let allocated = integerParts.reduce((a, b) => a + b, 0);
		const result = [...integerParts];

		while (allocated < totalBlocks) {
			let maxRemainder = -1;
			let maxIndex = -1;
			for (let i = 0; i < remainders.length; i++) {
				if (remainders[i] > maxRemainder) {
					maxRemainder = remainders[i];
					maxIndex = i;
				}
			}
			if (maxIndex === -1) break;
			result[maxIndex]++;
			remainders[maxIndex] = -1;
			allocated++;
		}

		return {
			working: result[0],
			permission: result[1],
			input: result[2],
			connecting: result[3]
		};
	});

	let statusArray = $derived.by(() => {
		const arr: string[] = [];
		for (let i = 0; i < blocks.working; i++) arr.push('working');
		for (let i = 0; i < blocks.permission; i++) arr.push('permission');
		for (let i = 0; i < blocks.input; i++) arr.push('input');
		for (let i = 0; i < blocks.connecting; i++) arr.push('connecting');
		while (arr.length < totalBlocks) arr.push('empty');
		return arr;
	});
</script>

<div class="system-status-bar">
	<div class="bar-header">
		<div class="pulse-indicator">
			<div class="dot"></div>
			<span class="label">SYSTEM_PULSE</span>
		</div>
		<div class="total-units">
			<span class="value">{total}</span>
			<span class="unit">ACTIVE_THREADS</span>
		</div>
	</div>

	<div class="progress-track" class:empty={total === 0} bind:clientWidth={trackWidth}>
		<div class="grid-container" style="grid-template-columns: repeat({columns}, 1fr);">
			{#each statusArray as status}
				<div class="rect {status}"></div>
			{/each}
		</div>
	</div>

	<div class="legend">
		<div class="legend-item" class:inactive={summary.working === 0}>
			<span class="dot working"></span>
			<span class="label">WORKING</span>
			<span class="count">{summary.working}</span>
		</div>
		<div class="legend-item" class:inactive={summary.permission === 0}>
			<span class="dot permission"></span>
			<span class="label">APPROVAL</span>
			<span class="count">{summary.permission}</span>
		</div>
		<div class="legend-item" class:inactive={summary.input === 0}>
			<span class="dot input"></span>
			<span class="label">WAITING</span>
			<span class="count">{summary.input}</span>
		</div>
		<div class="legend-item" class:inactive={summary.connecting === 0}>
			<span class="dot connecting"></span>
			<span class="label">CONNECTING</span>
			<span class="count">{summary.connecting}</span>
		</div>
	</div>

	<div class="deco-mesh"></div>
</div>

<style>
	.system-status-bar {
		position: relative;
		background: var(--bg-card);
		border: 1px solid var(--border-default);
		padding: var(--space-lg) var(--space-xl);
		display: flex;
		flex-direction: column;
		gap: var(--space-md);
		overflow: hidden;
		transition: border-color var(--transition-fast);
	}

	.system-status-bar:hover {
		border-color: var(--text-muted);
	}

	.bar-header {
		display: flex;
		justify-content: space-between;
		align-items: flex-end;
	}

	.pulse-indicator {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
	}

	.pulse-indicator .dot {
		width: 4px;
		height: 4px;
		background: var(--text-primary);
		animation: pulse-glow 2s linear infinite;
	}

	.pulse-indicator .label {
		font-family: var(--font-mono);
		font-size: 10px;
		color: var(--text-muted);
		letter-spacing: 0.2em;
	}

	.total-units {
		display: flex;
		align-items: baseline;
		gap: var(--space-sm);
	}

	.total-units .value {
		font-family: var(--font-pixel-grid);
		font-size: 24px;
		color: var(--text-primary);
		line-height: 1;
	}

	.total-units .unit {
		font-family: var(--font-mono);
		font-size: 9px;
		color: var(--text-muted);
		letter-spacing: 0.1em;
	}

	.progress-track {
		height: 28px;
		background: var(--bg-surface);
		border: 1px solid var(--border-default);
		position: relative;
		overflow: hidden;
		padding: 3px;
	}

	.progress-track.empty {
		opacity: 0.5;
	}

	.grid-container {
		display: grid;
		grid-template-rows: repeat(2, 1fr);
		gap: 3px;
		height: 100%;
	}

	.rect {
		width: 100%;
		height: 100%;
		background: rgba(255, 255, 255, 0.06);
		transition: background-color var(--transition-fast);
		border-radius: 1px;
	}

	.rect.working { background-color: var(--status-working); }
	.rect.permission { background-color: var(--status-permission); }
	.rect.input { background-color: var(--status-input); }
	.rect.connecting { background-color: var(--status-connecting); }

	.legend {
		display: flex;
		gap: var(--space-xl);
	}

	.legend-item {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		transition: transform var(--transition-fast);
	}

	.legend-item.inactive {
		opacity: 1; /* Force full opacity for readability */
	}

	.legend-item .dot {
		width: 8px;
		height: 8px;
	}

	.legend-item .dot.working { background: var(--status-working); }
	.legend-item .dot.permission { background: var(--status-permission); }
	.legend-item .dot.input { background: var(--status-input); }
	.legend-item .dot.connecting { background: var(--status-connecting); }

	.legend-item .label {
		font-family: var(--font-mono);
		font-size: 12px;
		color: var(--text-secondary);
		letter-spacing: 0.1em;
	}

	.legend-item .count {
		font-family: var(--font-pixel);
		font-size: 14px;
		color: var(--text-primary);
	}

	.deco-mesh {
		position: absolute;
		top: 0;
		right: 0;
		width: 100px;
		height: 100%;
		pointer-events: none;
		opacity: 0.05;
		background-image: 
			radial-gradient(var(--text-muted) 1px, transparent 1px);
		background-size: 4px 4px;
	}

	@keyframes pulse-glow {
		0%, 100% { opacity: 0.3; }
		50% { opacity: 1; }
	}
</style>
