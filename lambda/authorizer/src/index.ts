import type { APIGatewayRequestAuthorizerEventV2 } from "aws-lambda";
import { verifyToken } from "./jwt.js";
import { isAllowedIp } from "./vpn.js";

const vpnCidrs = (process.env.VPN_CIDR_BLOCKS ?? "")
  .split(",")
  .map((s) => s.trim())
  .filter(Boolean);

export async function handler(
  event: APIGatewayRequestAuthorizerEventV2
): Promise<{ isAuthorized: boolean }> {
  try {
    const authHeader = event.headers?.["authorization"] ?? "";

    if (!authHeader.toLowerCase().startsWith("bearer ")) {
      return { isAuthorized: false };
    }

    const token = authHeader.slice(7).trim();
    if (!token) return { isAuthorized: false };

    const sourceIp = event.requestContext.http.sourceIp;

    if (vpnCidrs.length > 0 && !isAllowedIp(sourceIp, vpnCidrs)) {
      return { isAuthorized: false };
    }

    await verifyToken(token);

    return { isAuthorized: true };
  } catch (err) {
    console.error("Authorizer error", err);
    return { isAuthorized: false };
  }
}
