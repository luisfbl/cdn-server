import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

const API_GATEWAY_URL = process.env.API_GATEWAY_URL || 'http://localstack:4566/restapis/3q5o86n2kp/prod/_user_request_/files';

export const GET: RequestHandler = async ({ url }) => {
  const queryString = url.search;
  const apiUrl = `${API_GATEWAY_URL}${queryString}`;

  try {
    const response = await fetch(apiUrl);
    const data = await response.json();

    if (data.files && Array.isArray(data.files)) {
      const documents = data.files.map((file: any) => ({
        id: file.pk || file.id,
        hash: file.checksum || file.hash,
        file_size: file.size,
        mime_type: file.contentType,
        created_at: file.processedAt,
        description: file.description
      }));

      return json(documents);
    }

    return json(data);
  } catch (error) {
    console.error('Error fetching from API Gateway:', error);
    return json({ error: 'Failed to fetch documents' }, { status: 500 });
  }
};

export const POST: RequestHandler = async ({ request }) => {
  try {
    const formData = await request.formData();
    const file = formData.get('file') as File;

    if (!file) {
      return json({ error: 'No file provided' }, { status: 400 });
    }

    const fileBuffer = await file.arrayBuffer();
    const response = await fetch(API_GATEWAY_URL, {
      method: 'POST',
      headers: {
        'Content-Type': file.type || 'application/octet-stream',
      },
      body: fileBuffer
    });

    const data = await response.json();

    return json(data, { status: response.status });
  } catch (error) {
    console.error('Error uploading to API Gateway:', error);
    return json({ error: 'Failed to upload file' }, { status: 500 });
  }
};
