function ipToInt(ip: string): number | null {
  const parts = ip.split(".");
  if (parts.length !== 4) return null;
  let result = 0;
  for (const part of parts) {
    const num = parseInt(part, 10);
    if (isNaN(num) || num < 0 || num > 255) return null;
    result = (result << 8) | num;
  }
  return result >>> 0;
}

export function isAllowedIp(sourceIp: string, cidrs: string[]): boolean {
  const ipInt = ipToInt(sourceIp);
  if (ipInt === null) return false;

  for (const cidr of cidrs) {
    const [network, prefixStr] = cidr.split("/");
    if (!network || !prefixStr) continue;
    const networkInt = ipToInt(network);
    if (networkInt === null) continue;
    const prefix = parseInt(prefixStr, 10);
    if (isNaN(prefix) || prefix < 1 || prefix > 32) continue;
    const mask = (~0 << (32 - prefix)) >>> 0;
    if ((ipInt & mask) === (networkInt & mask)) return true;
  }

  return false;
}
