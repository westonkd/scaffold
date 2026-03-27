import type { APIGatewayProxyEventV2, APIGatewayProxyResultV2 } from "aws-lambda";
import {
  S3Client,
  GetObjectCommand,
  PutObjectCommand,
  DeleteObjectCommand,
  ListObjectsV2Command,
} from "@aws-sdk/client-s3";

const s3 = new S3Client({});
const BUCKET = process.env.BUCKET_NAME!;

export async function handler(
  event: APIGatewayProxyEventV2
): Promise<APIGatewayProxyResultV2> {
  const method = event.requestContext.http.method.toUpperCase();
  const key = event.rawPath.replace(/^\//, "");

  try {
    if (method === "GET" && key === "") return handleList(event.queryStringParameters?.["prefix"] ?? "");
    if (method === "GET") return handleGet(key);
    if (method === "PUT") return handlePut(key, event);
    if (method === "DELETE") return handleDelete(key);

    return { statusCode: 405, body: "Method Not Allowed" };
  } catch (err) {
    console.error("S3 proxy error", { method, key, err });
    if (isNoSuchKey(err)) return { statusCode: 404, body: "Not Found" };
    return { statusCode: 500, body: "Internal Server Error" };
  }
}

async function handleGet(key: string): Promise<APIGatewayProxyResultV2> {
  const result = await s3.send(new GetObjectCommand({ Bucket: BUCKET, Key: key }));
  const bytes = await result.Body!.transformToByteArray();
  return {
    statusCode: 200,
    headers: { "Content-Type": result.ContentType ?? "application/octet-stream" },
    body: Buffer.from(bytes).toString("base64"),
    isBase64Encoded: true,
  };
}

async function handlePut(
  key: string,
  event: APIGatewayProxyEventV2
): Promise<APIGatewayProxyResultV2> {
  const body = event.isBase64Encoded
    ? Buffer.from(event.body ?? "", "base64")
    : Buffer.from(event.body ?? "", "utf-8");

  const tagging = event.headers?.["x-amz-tagging"];

  await s3.send(
    new PutObjectCommand({
      Bucket: BUCKET,
      Key: key,
      Body: body,
      ContentType: event.headers?.["content-type"],
      Tagging: tagging || undefined,
    })
  );

  return { statusCode: 200, body: "" };
}

async function handleDelete(key: string): Promise<APIGatewayProxyResultV2> {
  await s3.send(new DeleteObjectCommand({ Bucket: BUCKET, Key: key }));
  return { statusCode: 204, body: "" };
}

async function handleList(prefix: string): Promise<APIGatewayProxyResultV2> {
  const result = await s3.send(
    new ListObjectsV2Command({ Bucket: BUCKET, Prefix: prefix })
  );
  const keys = (result.Contents ?? []).map((obj) => obj.Key ?? "");
  return {
    statusCode: 200,
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ keys }),
  };
}

function isNoSuchKey(err: unknown): boolean {
  return (
    typeof err === "object" &&
    err !== null &&
    "name" in err &&
    (err.name === "NoSuchKey" || err.name === "NotFound")
  );
}
