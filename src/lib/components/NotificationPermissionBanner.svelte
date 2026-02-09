<script lang="ts">
	import { notificationPermission, requestNotificationPermission } from '../stores/sessions';

	let requesting = $state(false);

	async function handleRequestPermission() {
		requesting = true;
		try {
			await requestNotificationPermission();
		} finally {
			requesting = false;
		}
	}
</script>

{#if $notificationPermission !== 'granted'}
	<div class="banner">
		<div class="banner-content">
			<div class="banner-icon">🔔</div>
			<div class="banner-text">
				<div class="banner-title">Enable Notifications</div>
				<div class="banner-description">
					Get notified when sessions need permission or finish working
				</div>
			</div>
			<button class="banner-button" onclick={handleRequestPermission} disabled={requesting}>
				{requesting ? 'Requesting...' : 'Enable'}
			</button>
		</div>
	</div>
{/if}

<style>
	.banner {
		background: rgba(255, 255, 255, 0.05);
		border-bottom: 1px solid rgba(255, 255, 255, 0.1);
		padding: 12px 16px;
	}

	.banner-content {
		display: flex;
		align-items: center;
		gap: 12px;
	}

	.banner-icon {
		font-size: 24px;
		line-height: 1;
	}

	.banner-text {
		flex: 1;
		min-width: 0;
	}

	.banner-title {
		font-size: 14px;
		font-weight: 500;
		color: rgba(255, 255, 255, 0.9);
		margin-bottom: 2px;
	}

	.banner-description {
		font-size: 12px;
		color: rgba(255, 255, 255, 0.6);
	}

	.banner-button {
		background: rgba(255, 255, 255, 0.1);
		border: 1px solid rgba(255, 255, 255, 0.2);
		color: rgba(255, 255, 255, 0.9);
		padding: 6px 16px;
		border-radius: 6px;
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.2s ease;
		white-space: nowrap;
	}

	.banner-button:hover:not(:disabled) {
		background: rgba(255, 255, 255, 0.15);
		border-color: rgba(255, 255, 255, 0.3);
	}

	.banner-button:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}
</style>
