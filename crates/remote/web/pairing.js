/** Consume a QR credential locally; URL fragments are never sent in HTTP requests. */
export function consumeInvitation(location, history) {
  const params = new URLSearchParams(location.hash.slice(1));
  const invitation = params.get("pair");
  if (params.has("pair")) {
    try {
      history.replaceState(null, "", location.pathname + location.search);
    } catch {}
  }
  return /^[a-f0-9]{48}$/.test(invitation ?? "") ? invitation : null;
}
