output "api_endpoint" {
  value       = aws_apigatewayv2_stage.default.invoke_url
  description = "Base invoke URL for the API Gateway (use as api_gateway_url in ~/.scaffold/settings.json)"
}

output "api_id" {
  value       = aws_apigatewayv2_api.this.id
  description = "API Gateway ID"
}

output "authorizer_function_arn" {
  value       = aws_lambda_function.authorizer.arn
  description = "ARN of the Lambda authorizer function"
}

output "s3_proxy_role_arn" {
  value       = aws_iam_role.s3_proxy_lambda.arn
  description = "ARN of the S3 proxy Lambda execution role (add to bucket policy)"
}
