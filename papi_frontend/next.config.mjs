/** @type {import('next').NextConfig} */
const nextConfig = {
    experimental: {
      missingSuspenseWithCSRBailout: false, // TODO: fix this (https://nextjs.org/docs/messages/missing-suspense-with-csr-bailout)
    },
  };

export default nextConfig;
