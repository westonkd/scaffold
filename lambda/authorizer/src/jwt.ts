import { createRemoteJWKSet, jwtVerify } from "jose";

type JWKSClient = ReturnType<typeof createRemoteJWKSet>;
let jwks: JWKSClient | null = null;

export async function verifyToken(token: string): Promise<void> {
  if (!jwks) {
    const uri = process.env.JWKS_URI;
    if (!uri) {
      console.error("JWKS_URI environment variable is not set");
      throw new Error("JWKS_URI not configured");
    }
    jwks = createRemoteJWKSet(new URL(uri));
  }

  await jwtVerify(token, jwks, {
    issuer: process.env.JWT_ISSUER,
    audience: process.env.JWT_AUDIENCE,
  });
}
