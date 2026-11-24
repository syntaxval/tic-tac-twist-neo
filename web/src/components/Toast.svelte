<script lang="ts">
  interface Props {
    message: string | null;
    toastKey: number;
    onDismiss: () => void;
  }

  let { message, toastKey, onDismiss }: Props = $props();

  const toastId = 'settings-toast';
  let toastEl: HTMLDivElement | undefined = $state();
  let visible = $state(false);
  let liveAnnouncement = $state('');
  let focusInside = false;
  let timer: ReturnType<typeof setTimeout> | null = null;
  let priorFocus: HTMLElement | null = null;

  function clearTimer() {
    if (timer !== null) {
      clearTimeout(timer);
      timer = null;
    }
  }

  function scheduleDismiss() {
    clearTimer();
    timer = setTimeout(() => {
      if (!focusInside) {
        dismiss(false);
      }
    }, 3200);
  }

  function dismiss(restoreFocus: boolean) {
    clearTimer();
    if (toastEl && typeof toastEl.hidePopover === 'function') {
      try {
        if (toastEl.matches(':popover-open')) {
          toastEl.hidePopover();
        }
      } catch {
        // Ignore if already closed.
      }
    }
    visible = false;
    onDismiss();
    if (restoreFocus && priorFocus && priorFocus.isConnected) {
      priorFocus.focus();
    }
  }

  function announce(text: string) {
    const doc = document as Document & { ariaNotify?: (msg: string) => void };
    if (typeof doc.ariaNotify === 'function') {
      doc.ariaNotify(text);
    } else {
      liveAnnouncement = '';
      queueMicrotask(() => {
        liveAnnouncement = text;
      });
    }
  }

  $effect(() => {
    const key = toastKey;
    const msg = message;
    if (key === 0 || !msg || !toastEl) return;

    priorFocus =
      document.activeElement instanceof HTMLElement
        ? document.activeElement
        : null;
    visible = true;

    if (typeof toastEl.showPopover === 'function') {
      try {
        if (!toastEl.matches(':popover-open')) {
          toastEl.showPopover();
        }
      } catch {
        // Ignore if already open.
      }
    }

    announce(msg);
    scheduleDismiss();

    return () => {
      clearTimer();
    };
  });

  function handleFocusIn() {
    focusInside = true;
    clearTimer();
  }

  function handleFocusOut(e: FocusEvent) {
    if (toastEl && !toastEl.contains(e.relatedTarget as Node | null)) {
      focusInside = false;
      if (visible) {
        scheduleDismiss();
      }
    }
  }

  function handleCloseClick() {
    dismiss(true);
  }
</script>

<div role="region" aria-label="Notifications" class="toast-region">
  <p class="sr-only">Screen reader announcements</p>
  <div class="sr-only" aria-live="polite">{liveAnnouncement}</div>

  <div
    bind:this={toastEl}
    id={toastId}
    popover="manual"
    class="toast"
    class:is-open={visible}
    onfocusin={handleFocusIn}
    onfocusout={handleFocusOut}
  >
    <span class="check" aria-hidden="true">✓</span>
    <span class="msg">{message ?? ''}</span>
    <button
      type="button"
      class="close"
      popovertarget={toastId}
      popovertargetaction="hide"
      aria-label="Close notification"
      onclick={handleCloseClick}
    >
      ×
    </button>
  </div>
</div>

<style>
  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip-path: inset(50%);
    white-space: nowrap;
    border: 0;
  }

  .toast {
    position: fixed;
    inset: auto 1.25rem 1.25rem auto;
    margin: 0;
    display: none;
    align-items: center;
    gap: 0.6rem;
    max-width: min(24rem, calc(100vw - 2rem));
    padding: 0.65rem 0.85rem;
    border-radius: 0.75rem;
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text);
    box-shadow: 0 12px 28px rgb(0 0 0 / 0.22);
    font-size: 0.875rem;
    line-height: 1.35;
    opacity: 0;
    transform: translateY(0.5rem);
    transition:
      opacity 180ms ease,
      transform 180ms ease,
      display 180ms ease,
      overlay 180ms ease;
    transition-behavior: allow-discrete;
  }

  .toast:is(:popover-open, .is-open) {
    display: flex;
    opacity: 1;
    transform: translateY(0);
  }

  @starting-style {
    .toast:is(:popover-open, .is-open) {
      opacity: 0;
      transform: translateY(0.5rem);
    }
  }

  .check {
    display: grid;
    place-items: center;
    width: 1.35rem;
    height: 1.35rem;
    border-radius: 999px;
    background: color-mix(in srgb, #22c55e 18%, transparent);
    color: #22c55e;
    font-weight: 700;
    flex-shrink: 0;
  }

  .msg {
    flex: 1;
    min-width: 0;
    overflow-wrap: break-word;
  }

  .close {
    display: grid;
    place-items: center;
    width: 1.6rem;
    height: 1.6rem;
    padding: 0;
    border: none;
    border-radius: 0.4rem;
    background: transparent;
    color: var(--muted);
    font-size: 1.15rem;
    line-height: 1;
    flex-shrink: 0;
  }

  .close:hover {
    background: var(--surface-alt);
    color: var(--text);
  }

  @media (max-width: 640px) {
    .toast {
      inset: auto 1rem 1rem 1rem;
      margin-inline: auto;
      max-width: calc(100vw - 2rem);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .toast {
      transition: none;
      transform: none;
    }

    @starting-style {
      .toast:is(:popover-open, .is-open) {
        transform: none;
      }
    }
  }
</style>
