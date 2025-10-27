import type { RequestHandler } from './$types';

const API_GATEWAY_URL = process.env.API_GATEWAY_URL || 'http://localstack:4566/restapis/3q5o86n2kp/prod/_user_request_/files';

export const GET: RequestHandler = async ({ params }) => {
  const { id } = params;

  try {
    const response = await fetch(`${API_GATEWAY_URL}/${id}`);

    if (!response.ok) {
      return new Response(JSON.stringify({ error: 'File not found' }), {
        status: response.status,
        headers: { 'Content-Type': 'application/json' }
      });
    }

    const fileData = await response.arrayBuffer();
    const contentType = response.headers.get('Content-Type') || 'application/octet-stream';

    return new Response(fileData, {
      status: 200,
      headers: {
        'Content-Type': contentType,
        'Cache-Control': 'public, max-age=31536000'
      }
    });
  } catch (error) {
    console.error('Error fetching file from API Gateway:', error);
    return new Response(JSON.stringify({ error: 'Failed to fetch file' }), {
      status: 500,
      headers: { 'Content-Type': 'application/json' }
    });
  }
};
